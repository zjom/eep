//! Parsing and formatting durations.

fn unit_seconds(unit: &str) -> Option<f64> {
    Some(match unit {
        "s" | "sec" | "secs" | "second" | "seconds" => 1.0,
        "m" | "min" | "mins" | "minute" | "minutes" => 60.0,
        "h" | "hr" | "hrs" | "hour" | "hours" => 3600.0,
        "d" | "day" | "days" => 86400.0,
        _ => return None,
    })
}

/// `90` -> 90s, `1h30m`, `2.5h`, `10 min`; `inf`/`forever` -> `None`.
pub fn parse(text: &str) -> Result<Option<f64>, String> {
    let t = text.trim().to_lowercase();
    if matches!(t.as_str(), "inf" | "infinity" | "forever") {
        return Ok(None);
    }
    let bad = || format!("can't parse duration '{text}' (try 90, 45s, 20m, 1h30m, inf)");
    let b = t.as_bytes();
    let mut i = 0;
    let number = |i: &mut usize| -> Option<f64> {
        let start = *i;
        while *i < b.len() && b[*i].is_ascii_digit() {
            *i += 1;
        }
        if *i == start {
            return None;
        }
        if *i + 1 < b.len() && b[*i] == b'.' && b[*i + 1].is_ascii_digit() {
            *i += 1;
            while *i < b.len() && b[*i].is_ascii_digit() {
                *i += 1;
            }
        }
        t[start..*i].parse().ok()
    };

    if let Some(n) = number(&mut i)
        && i == b.len()
    {
        return Ok(Some(n));
    }

    i = 0;
    let mut total = 0.0;
    let mut parts = 0;
    loop {
        while i < b.len() && b[i].is_ascii_whitespace() {
            i += 1;
        }
        if i == b.len() {
            break;
        }
        let n = number(&mut i).ok_or_else(bad)?;
        while i < b.len() && b[i].is_ascii_whitespace() {
            i += 1;
        }
        let start = i;
        while i < b.len() && b[i].is_ascii_lowercase() {
            i += 1;
        }
        if i == start {
            return Err(bad());
        }
        let unit = &t[start..i];
        let secs =
            unit_seconds(unit).ok_or_else(|| format!("unknown time unit '{unit}' in '{text}'"))?;
        total += n * secs;
        parts += 1;
    }
    if parts == 0 {
        return Err(bad());
    }
    Ok(Some(total))
}

pub fn fmt_clock(secs: f64) -> String {
    let s = secs.max(0.0) as u64;
    let (h, m, s) = (s / 3600, s % 3600 / 60, s % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m:02}:{s:02}")
    }
}

pub fn fmt_human(secs: f64) -> String {
    let mut s = secs.round().max(0.0) as u64;
    let mut out = Vec::new();
    for (unit, size) in [("d", 86400), ("h", 3600), ("m", 60), ("s", 1)] {
        if s >= size || (unit == "s" && out.is_empty()) {
            out.push(format!("{}{unit}", s / size));
            s %= size;
        }
    }
    out.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses() {
        assert_eq!(parse("90"), Ok(Some(90.0)));
        assert_eq!(parse(" 2.5 "), Ok(Some(2.5)));
        assert_eq!(parse("45s"), Ok(Some(45.0)));
        assert_eq!(parse("20m"), Ok(Some(1200.0)));
        assert_eq!(parse("1h30m"), Ok(Some(5400.0)));
        assert_eq!(parse("2.5h"), Ok(Some(9000.0)));
        assert_eq!(parse("10 min"), Ok(Some(600.0)));
        assert_eq!(parse("1H 30 Mins"), Ok(Some(5400.0)));
        assert_eq!(parse("inf"), Ok(None));
        assert_eq!(parse("Forever"), Ok(None));
    }

    #[test]
    fn rejects() {
        for bad in ["", "abc", "1h30", "1.5.3h", "-5", "5m x", "1."] {
            assert!(parse(bad).is_err(), "{bad:?} should fail");
        }
        assert_eq!(
            parse("3 weeks"),
            Err("unknown time unit 'weeks' in '3 weeks'".into())
        );
    }

    #[test]
    fn formats() {
        assert_eq!(fmt_clock(65.9), "01:05");
        assert_eq!(fmt_clock(3600.0), "1:00:00");
        assert_eq!(fmt_clock(-3.0), "00:00");
        assert_eq!(fmt_human(0.2), "0s");
        assert_eq!(fmt_human(5400.0), "1h 30m");
        assert_eq!(fmt_human(90061.0), "1d 1h 1m 1s");
    }
}
