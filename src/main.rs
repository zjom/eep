//! eep - keep the computer awake for a while, with something nice to look at.
//!
//! Works like `caffeinate -t` (macOS) or `systemd-inhibit sleep` (Linux): it
//! holds a sleep inhibitor for the given duration (or until you quit), and
//! meanwhile plays a terminal animation.

mod anim;
mod cli;
mod color;
mod duration;
mod inhibit;
mod rng;
mod term;

use std::io::Write;
use std::time::Instant;

use anim::{ALL, Animation, Clock};
use inhibit::Inhibitor;

const DIED: &str = "the sleep inhibitor process died unexpectedly";

/// How a run ended: Ok(exit code), or Err(message) if the inhibitor failed.
type Outcome = Result<i32, String>;

fn main() {
    std::process::exit(real_main());
}

fn real_main() -> i32 {
    let args = match cli::parse(std::env::args().skip(1)) {
        Ok(cli::Parsed::Run(a)) => a,
        Ok(cli::Parsed::Help) => {
            println!("{}", cli::help());
            return 0;
        }
        Err(e) => {
            eprintln!("{}\neep: error: {e}", cli::USAGE);
            return 2;
        }
    };

    if args.list {
        let width = ALL.iter().map(|a| a.name.len()).max().unwrap_or(0);
        for a in ALL {
            println!("  {:<width$}  {}", a.name, a.blurb);
        }
        return 0;
    }

    let start = anim::index_of(&args.animation).unwrap_or_else(|| rng::Rng::new().below(ALL.len()));
    let animate = !args.no_animation && term::isatty(libc::STDOUT_FILENO);

    term::install_signal_handlers();

    let mut inh = match Inhibitor::start(&args.why, args.display) {
        Ok(i) => i,
        Err(e) => {
            eprintln!("eep: {e}");
            return 1;
        }
    };
    let clock = Clock::new(args.duration);
    let outcome = if animate {
        run_animated(start, &clock, args.fps.clamp(1, 120) as u32, &mut inh)
    } else {
        let until = match args.duration {
            Some(d) => format!(
                "for {} (until {})",
                duration::fmt_human(d),
                term::local_hhmm_in(d)
            ),
            None => "until interrupted".into(),
        };
        let mut out = std::io::stdout();
        let _ = writeln!(out, "eep: holding {} {until}. ctrl-c to stop.", inh.backend);
        let _ = out.flush();
        run_plain(&clock, &mut inh)
    };
    drop(inh); // release the lock before reporting

    match outcome {
        Ok(code) => {
            let _ = writeln!(
                std::io::stdout(),
                "eep: kept awake for {}; sleep allowed again.",
                duration::fmt_human(clock.elapsed())
            );
            code
        }
        Err(e) => {
            eprintln!("eep: {e}");
            1
        }
    }
}

/// Check the things that end a run; `Some` means stop now.
fn should_stop(clock: &Clock, inh: &mut Inhibitor) -> Option<Outcome> {
    if let Some(sig) = term::pending_signal() {
        return Some(Ok(128 + sig));
    }
    if clock.remaining().is_some_and(|r| r <= 0.0) {
        return Some(Ok(0));
    }
    if !inh.alive() {
        return Some(Err(DIED.into()));
    }
    None
}

fn run_plain(clock: &Clock, inh: &mut Inhibitor) -> Outcome {
    loop {
        if let Some(done) = should_stop(clock, inh) {
            return done;
        }
        term::wait(None, clock.remaining().map_or(1.0, |r| r.min(1.0)));
    }
}

/// Alternate screen with hidden cursor and no autowrap, undone on drop.
struct Screen;

impl Screen {
    fn enter() -> Self {
        let mut out = std::io::stdout();
        let _ = out.write_all(b"\x1b[?1049h\x1b[?25l\x1b[?7l\x1b[2J");
        let _ = out.flush();
        Screen
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        let mut out = std::io::stdout();
        let _ = out.write_all(b"\x1b[0m\x1b[?7h\x1b[?25h\x1b[?1049l");
        let _ = out.flush();
    }
}

fn run_animated(start: usize, clock: &Clock, fps: u32, inh: &mut Inhibitor) -> Outcome {
    let keys = term::isatty(libc::STDIN_FILENO);
    let _cbreak = if keys {
        term::Cbreak::enter(libc::STDIN_FILENO)
    } else {
        None
    };
    let _screen = Screen::enter();
    let mut out = std::io::stdout();

    let mut idx = start;
    let mut size = (0, 0);
    let mut anim: Option<Box<dyn Animation>> = None;
    let frame_time = 1.0 / f64::from(fps);
    let mut last = Instant::now();
    let mut show_fps = false;
    let mut fps_label = String::new();
    let (mut fps_count, mut fps_since) = (0u32, last);
    let mut keybuf = [0u8; 64];

    loop {
        if let Some(done) = should_stop(clock, inh) {
            return done;
        }

        let (cols, rows) = term::size();
        let (cols, rows) = (cols.max(1), rows.max(1));
        let mut buf = String::new();
        match anim.as_mut() {
            Some(_) if size == (cols, rows) => {}
            Some(a) => {
                a.resize(cols, rows);
                buf.push_str("\x1b[2J");
            }
            None => {
                anim = Some((ALL[idx].make)(cols, rows));
                buf.push_str("\x1b[2J");
            }
        }
        size = (cols, rows);

        let now = Instant::now();
        let dt = (now - last).as_secs_f64().min(0.1);
        last = now;
        let lines = anim.as_mut().unwrap().frame(dt, clock);
        buf.push_str("\x1b[?2026h"); // synchronized output (ignored if unsupported)
        for (y, line) in lines.iter().take(rows).enumerate() {
            buf.push_str(&format!("\x1b[{};1H\x1b[0m{line}", y + 1));
        }
        if show_fps && !fps_label.is_empty() {
            let label: String = fps_label.chars().take(cols).collect();
            let col = (cols + 1).saturating_sub(fps_label.chars().count()).max(1);
            buf.push_str(&format!("\x1b[1;{col}H\x1b[0;7m{label}"));
        }
        buf.push_str("\x1b[0m\x1b[?2026l");
        let _ = out.write_all(buf.as_bytes());
        let _ = out.flush();

        // measured rate, including time blocked writing to the terminal
        fps_count += 1;
        let span = fps_since.elapsed().as_secs_f64();
        if span >= 0.5 {
            fps_label = format!(" {:.0}/{fps} fps ", f64::from(fps_count) / span);
            fps_count = 0;
            fps_since = Instant::now();
        }

        let mut wait = (frame_time - now.elapsed().as_secs_f64()).max(0.0);
        if let Some(rem) = clock.remaining() {
            wait = wait.min(rem);
        }
        if !term::wait(keys.then_some(libc::STDIN_FILENO), wait) {
            continue;
        }
        let data = term::read(libc::STDIN_FILENO, &mut keybuf);
        let has = |seq: &[u8]| data.windows(seq.len()).any(|w| w == seq);
        match data {
            b"q" | b"Q" | b"\x1b" => return Ok(0),
            b"f" | b"F" => {
                show_fps = !show_fps;
                if !show_fps {
                    let _ = out.write_all(b"\x1b[2J");
                }
            }
            _ => {}
        }
        let step = if matches!(data, b"n" | b"N" | b" ") || has(b"\x1b[C") {
            1
        } else if matches!(data, b"p" | b"P") || has(b"\x1b[D") {
            ALL.len() - 1
        } else {
            0
        };
        if step != 0 {
            idx = (idx + step) % ALL.len();
            anim = Some((ALL[idx].make)(cols, rows));
        }
    }
}
