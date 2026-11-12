//! Command-line parsing (argparse-compatible enough for eep's options).

use crate::anim;
use crate::duration;

#[derive(Debug)]
pub struct Args {
    pub duration: Option<f64>,
    pub animation: String,
    pub no_animation: bool,
    pub list: bool,
    pub display: bool,
    pub fps: i64,
    pub why: String,
}

pub enum Parsed {
    Run(Args),
    Help,
}

pub const USAGE: &str =
    "usage: eep [-h] [-a NAME] [-n] [-l] [-d] [--fps FPS] [--why WHY] [duration]";

pub fn help() -> String {
    let names: Vec<_> = anim::ALL.iter().map(|a| a.name).collect();
    format!(
        "{USAGE}

Keep the computer from sleeping for a while, with terminal animations.

positional arguments:
  duration              how long to stay awake (default: until you quit)

options:
  -h, --help            show this help message and exit
  -a, --animation NAME  one of: {}, random (default)
  -n, --no-animation    just inhibit sleep quietly, no animation
  -l, --list            list animations and exit
  -d, --display         also keep the display awake (macOS: caffeinate -d;
                        Linux: adds an idle inhibitor)
  --fps FPS             animation frame rate (default 20)
  --why WHY             reason shown to the OS

durations: 90, 45s, 20m, 1h30m, 2.5h, '10 min', inf (default: until you quit)
keys while animating: n/space/→ next animation, p/← previous, f fps meter, q/esc quit

examples:
  eep 30m
  eep 2h -a fire
  eep 1h --no-animation
  eep -a random",
        names.join(", ")
    )
}

const LONG: [&str; 7] = [
    "--help",
    "--animation",
    "--no-animation",
    "--list",
    "--display",
    "--fps",
    "--why",
];

/// Resolve a (possibly abbreviated) long option, as argparse does.
fn long_option(arg: &str) -> Result<&'static str, String> {
    if let Some(exact) = LONG.iter().find(|&&o| o == arg) {
        return Ok(exact);
    }
    let hits: Vec<_> = LONG.iter().filter(|o| o.starts_with(arg)).collect();
    match hits.as_slice() {
        [one] => Ok(one),
        [] => Err(format!("unrecognized arguments: {arg}")),
        many => Err(format!(
            "ambiguous option: {arg} could match {}",
            many.iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
}

pub fn parse(argv: impl IntoIterator<Item = String>) -> Result<Parsed, String> {
    let mut a = Args {
        duration: None,
        animation: "random".into(),
        no_animation: false,
        list: false,
        display: false,
        fps: 20,
        why: "eep is keeping things awake".into(),
    };
    let mut duration: Option<String> = None;
    let mut argv = argv.into_iter();
    let mut only_positional = false;

    // Takes the value for `opt`: the inline part if any, else the next arg.
    let value = |opt: &str, inline: Option<String>, argv: &mut dyn Iterator<Item = String>| {
        inline
            .or_else(|| argv.next())
            .ok_or_else(|| format!("argument {opt}: expected one argument"))
    };

    while let Some(arg) = argv.next() {
        if only_positional || arg == "-" || !arg.starts_with('-') {
            if duration.is_some() {
                return Err(format!("unrecognized arguments: {arg}"));
            }
            duration = Some(arg);
        } else if arg == "--" {
            only_positional = true;
        } else if let Some(body) = arg.strip_prefix("--") {
            let (name, inline) = match body.split_once('=') {
                Some((n, v)) => (format!("--{n}"), Some(v.to_string())),
                None => (arg.clone(), None),
            };
            match long_option(&name)? {
                "--help" => return Ok(Parsed::Help),
                "--animation" => a.animation = value("-a/--animation", inline, &mut argv)?,
                "--fps" => a.fps = parse_fps(&value("--fps", inline, &mut argv)?)?,
                "--why" => a.why = value("--why", inline, &mut argv)?,
                flag => {
                    if inline.is_some() {
                        return Err(format!("argument {flag}: ignored explicit argument"));
                    }
                    match flag {
                        "--no-animation" => a.no_animation = true,
                        "--list" => a.list = true,
                        _ => a.display = true,
                    }
                }
            }
        } else {
            // Cluster of short flags, e.g. `-nd` or `-afire`.
            let mut rest = &arg[1..];
            while let Some(c) = rest.chars().next() {
                rest = &rest[c.len_utf8()..];
                match c {
                    'h' => return Ok(Parsed::Help),
                    'n' => a.no_animation = true,
                    'l' => a.list = true,
                    'd' => a.display = true,
                    'a' => {
                        let inline =
                            (!rest.is_empty()).then(|| rest.trim_start_matches('=').to_string());
                        a.animation = value("-a/--animation", inline, &mut argv)?;
                        break;
                    }
                    _ => return Err(format!("unrecognized arguments: {arg}")),
                }
            }
        }
    }

    let valid = anim::index_of(&a.animation).is_some() || a.animation == "random";
    if !valid {
        let mut names: Vec<_> = anim::ALL.iter().map(|x| x.name).collect();
        names.push("random");
        return Err(format!(
            "argument -a/--animation: invalid choice: '{}' (choose from {})",
            a.animation,
            names.join(", ")
        ));
    }
    if let Some(d) = duration {
        a.duration = duration::parse(&d).map_err(|e| format!("argument duration: {e}"))?;
    }
    Ok(Parsed::Run(a))
}

fn parse_fps(v: &str) -> Result<i64, String> {
    v.trim()
        .parse()
        .map_err(|_| format!("argument --fps: invalid int value: '{v}'"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(args: &[&str]) -> Result<Args, String> {
        match parse(args.iter().map(|s| s.to_string()))? {
            Parsed::Run(a) => Ok(a),
            Parsed::Help => Err("help".into()),
        }
    }

    #[test]
    fn options() {
        let a = run(&["2h", "-a", "fire", "--fps=30", "-nd", "--why", "build"]).unwrap();
        assert_eq!(a.duration, Some(7200.0));
        assert_eq!(a.animation, "fire");
        assert_eq!(a.fps, 30);
        assert!(a.no_animation && a.display && !a.list);
        assert_eq!(a.why, "build");

        assert_eq!(run(&["-apipes"]).unwrap().animation, "pipes");
        assert_eq!(run(&["--anim", "life"]).unwrap().animation, "life");
        assert_eq!(run(&["10 min"]).unwrap().duration, Some(600.0));
        assert_eq!(run(&[]).unwrap().duration, None);
        assert!(run(&["--list"]).unwrap().list);
    }

    #[test]
    fn errors() {
        assert!(run(&["-a", "nope"]).unwrap_err().contains("invalid choice"));
        assert!(run(&["--fps", "x"]).unwrap_err().contains("invalid int"));
        assert!(run(&["1h", "2h"]).unwrap_err().contains("unrecognized"));
        assert!(run(&["--x"]).unwrap_err().contains("unrecognized"));
        assert!(
            run(&["--display=1"])
                .unwrap_err()
                .contains("ignored explicit")
        );
        assert!(run(&["-a"]).unwrap_err().contains("expected one argument"));
        assert!(run(&["xyz"]).unwrap_err().starts_with("argument duration"));
        assert_eq!(run(&["-h"]).unwrap_err(), "help");
    }
}
