//! Colours and ANSI cell encoding.

use std::fmt::Write;
use std::sync::OnceLock;

pub type Rgb = (u8, u8, u8);

fn truecolor() -> bool {
    static TC: OnceLock<bool> = OnceLock::new();
    *TC.get_or_init(|| {
        let v = std::env::var("COLORTERM")
            .unwrap_or_default()
            .to_lowercase();
        v == "truecolor" || v == "24bit"
    })
}

pub fn to256((r, g, b): Rgb) -> u8 {
    let (r, g, b) = (i32::from(r), i32::from(g), i32::from(b));
    if r.max(g).max(b) - r.min(g).min(b) < 12 {
        // near-grey: use the grey ramp
        if r < 8 {
            return 16;
        }
        if r > 243 {
            return 231;
        }
        return 232 + ((f64::from(r - 8) / 10.0).round() as u8).min(23);
    }
    let q = |v: i32| {
        if v < 48 {
            0
        } else if v < 115 {
            1
        } else {
            (v - 35) / 40
        }
    };
    (16 + 36 * q(r) + 6 * q(g) + q(b)) as u8
}

fn push_color(out: &mut String, c: Option<Rgb>, base: u8) {
    match c {
        None => write!(out, "\x1b[{}m", base + 9),
        Some(c) if truecolor() => write!(out, "\x1b[{};2;{};{};{}m", base + 8, c.0, c.1, c.2),
        Some(c) => write!(out, "\x1b[{};5;{}m", base + 8, to256(c)),
    }
    .unwrap();
}

pub fn fg(c: Option<Rgb>) -> String {
    let mut s = String::new();
    push_color(&mut s, c, 30);
    s
}

/// Turn parallel slices of chars/colours into one ANSI string, emitting escape
/// codes only when the colour actually changes (and only on visible cells).
pub fn encode(chars: &[char], fgs: &[Option<Rgb>]) -> String {
    let mut out = String::with_capacity(chars.len() * 4);
    let mut last: Option<Option<Rgb>> = None;
    for (&ch, &f) in chars.iter().zip(fgs) {
        if ch != ' ' && last != Some(f) {
            push_color(&mut out, f, 30);
            last = Some(f);
        }
        out.push(ch);
    }
    out
}

/// Like `encode`, with a background colour per cell too.
pub fn encode_bg(chars: &[char], fgs: &[Rgb], bgs: &[Rgb]) -> String {
    let mut out = String::with_capacity(chars.len() * 24);
    let (mut lf, mut lb) = (None, None);
    for ((&ch, &f), &b) in chars.iter().zip(fgs).zip(bgs) {
        if lf != Some(f) {
            push_color(&mut out, Some(f), 30);
            lf = Some(f);
        }
        if lb != Some(b) {
            push_color(&mut out, Some(b), 40);
            lb = Some(b);
        }
        out.push(ch);
    }
    out.push_str("\x1b[49m");
    out
}

pub fn hsv(h: f64, s: f64, v: f64) -> Rgb {
    let h = h.rem_euclid(1.0);
    let (r, g, b) = if s == 0.0 {
        (v, v, v)
    } else {
        let i = (h * 6.0).floor();
        let f = h * 6.0 - i;
        let (p, q, t) = (v * (1.0 - s), v * (1.0 - s * f), v * (1.0 - s * (1.0 - f)));
        match i as i32 % 6 {
            0 => (v, t, p),
            1 => (q, v, p),
            2 => (p, v, t),
            3 => (p, q, v),
            4 => (t, p, v),
            _ => (v, p, q),
        }
    };
    ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

/// Linear interpolation through `[(pos, rgb), ...]` into `n` colours.
pub fn gradient(stops: &[(f64, Rgb)], n: usize) -> Vec<Rgb> {
    let lerp = |a: u8, b: u8, f: f64| (f64::from(a) + (f64::from(b) - f64::from(a)) * f) as u8;
    (0..n)
        .map(|i| {
            let p = i as f64 / (n - 1) as f64;
            stops
                .windows(2)
                .find(|w| p <= w[1].0)
                .map(|w| {
                    let ((p0, c0), (p1, c1)) = (w[0], w[1]);
                    let f = if p1 > p0 { (p - p0) / (p1 - p0) } else { 0.0 };
                    (
                        lerp(c0.0, c1.0, f),
                        lerp(c0.1, c1.1, f),
                        lerp(c0.2, c1.2, f),
                    )
                })
                .unwrap_or(stops[stops.len() - 1].1)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_256() {
        assert_eq!(to256((0, 0, 0)), 16);
        assert_eq!(to256((255, 255, 255)), 231);
        assert_eq!(to256((128, 128, 128)), 244);
        assert_eq!(to256((255, 0, 0)), 196);
        assert_eq!(to256((0, 255, 0)), 46);
    }

    #[test]
    fn hsv_primaries() {
        assert_eq!(hsv(0.0, 1.0, 1.0), (255, 0, 0));
        assert_eq!(hsv(1.0 / 3.0, 1.0, 1.0), (0, 255, 0));
        assert_eq!(hsv(-1.0 / 3.0, 1.0, 1.0), (0, 0, 255));
    }

    #[test]
    fn gradient_endpoints() {
        let g = gradient(&[(0.0, (0, 0, 0)), (1.0, (200, 100, 50))], 5);
        assert_eq!(g.len(), 5);
        assert_eq!(g[0], (0, 0, 0));
        assert_eq!(g[4], (200, 100, 50));
    }
}
