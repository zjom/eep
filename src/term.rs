//! Terminal plumbing: size, cbreak mode, signals, interruptible waits.

use std::sync::atomic::{AtomicI32, Ordering};

pub fn isatty(fd: i32) -> bool {
    // SAFETY: isatty only inspects the descriptor.
    unsafe { libc::isatty(fd) == 1 }
}

/// (columns, rows), honouring COLUMNS/LINES like Python's shutil does.
pub fn size() -> (usize, usize) {
    let env = |k| {
        std::env::var(k)
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(0)
    };
    let (mut cols, mut rows) = (env("COLUMNS"), env("LINES"));
    if cols == 0 || rows == 0 {
        // SAFETY: TIOCGWINSZ fills a winsize struct we own.
        let mut ws: libc::winsize = unsafe { std::mem::zeroed() };
        if unsafe { libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut ws) } == 0 {
            if cols == 0 {
                cols = usize::from(ws.ws_col);
            }
            if rows == 0 {
                rows = usize::from(ws.ws_row);
            }
        }
    }
    (
        if cols == 0 { 80 } else { cols },
        if rows == 0 { 24 } else { rows },
    )
}

/// Puts a tty into cbreak mode (no echo, no line buffering; signals still work)
/// and restores the previous settings on drop.
pub struct Cbreak {
    fd: i32,
    saved: libc::termios,
}

impl Cbreak {
    pub fn enter(fd: i32) -> Option<Self> {
        // SAFETY: tcgetattr/tcsetattr on a termios struct we own.
        unsafe {
            let mut saved: libc::termios = std::mem::zeroed();
            if libc::tcgetattr(fd, &mut saved) != 0 {
                return None;
            }
            let mut t = saved;
            t.c_lflag &= !(libc::ICANON | libc::ECHO);
            t.c_cc[libc::VMIN] = 1;
            t.c_cc[libc::VTIME] = 0;
            libc::tcsetattr(fd, libc::TCSAFLUSH, &t);
            Some(Cbreak { fd, saved })
        }
    }
}

impl Drop for Cbreak {
    fn drop(&mut self) {
        // SAFETY: restoring the settings saved in `enter`.
        unsafe { libc::tcsetattr(self.fd, libc::TCSADRAIN, &self.saved) };
    }
}

static SIGNAL: AtomicI32 = AtomicI32::new(0);

extern "C" fn on_signal(sig: libc::c_int) {
    SIGNAL.store(sig, Ordering::SeqCst);
}

/// Record SIGINT/SIGTERM/SIGHUP instead of dying, so the main loop can restore
/// the terminal and release the inhibitor. No SA_RESTART: waits get EINTR.
pub fn install_signal_handlers() {
    for sig in [libc::SIGINT, libc::SIGTERM, libc::SIGHUP] {
        // SAFETY: the handler only stores to an atomic.
        unsafe {
            let mut sa: libc::sigaction = std::mem::zeroed();
            sa.sa_sigaction = on_signal as *const () as libc::sighandler_t;
            libc::sigemptyset(&mut sa.sa_mask);
            libc::sigaction(sig, &sa, std::ptr::null_mut());
        }
    }
}

pub fn pending_signal() -> Option<i32> {
    match SIGNAL.load(Ordering::SeqCst) {
        0 => None,
        s => Some(s),
    }
}

/// Wait up to `secs` for `fd` to become readable (or just sleep if `fd` is
/// None). Returns early on a signal. True if `fd` is readable.
pub fn wait(fd: Option<i32>, secs: f64) -> bool {
    let ms = (secs.max(0.0) * 1000.0).round().min(f64::from(i32::MAX)) as i32;
    let mut pfd = libc::pollfd {
        fd: fd.unwrap_or(-1),
        events: libc::POLLIN,
        revents: 0,
    };
    let n = if fd.is_some() { 1 } else { 0 };
    // SAFETY: one pollfd we own (or none).
    let r = unsafe { libc::poll(&mut pfd, n, ms) };
    r > 0 && pfd.revents & libc::POLLIN != 0
}

pub fn read(fd: i32, buf: &mut [u8]) -> &[u8] {
    // SAFETY: reading into a buffer we own, bounded by its length.
    let n = unsafe { libc::read(fd, buf.as_mut_ptr().cast(), buf.len()) };
    &buf[..n.max(0) as usize]
}

/// Local wall-clock "HH:MM", `secs` from now.
pub fn local_hhmm_in(secs: f64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0);
    let t = now + secs;
    if !t.is_finite() || t > 1e15 {
        return "the heat death of the universe".into();
    }
    let t = t as libc::time_t;
    // SAFETY: localtime_r writes into a tm we own.
    unsafe {
        let mut tm: libc::tm = std::mem::zeroed();
        if libc::localtime_r(&t, &mut tm).is_null() {
            return "?".into();
        }
        format!("{:02}:{:02}", tm.tm_hour, tm.tm_min)
    }
}
