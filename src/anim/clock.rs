use super::{Animation, Clock};
use crate::color::{Rgb, fg, hsv};
use crate::duration::fmt_clock;
use crate::rng::Rng;

fn glyph(c: char) -> [&'static str; 5] {
    match c {
        '0' => ["███", "█ █", "█ █", "█ █", "███"],
        '1' => [" █ ", "██ ", " █ ", " █ ", "███"],
        '2' => ["███", "  █", "███", "█  ", "███"],
        '3' => ["███", "  █", "███", "  █", "███"],
        '4' => ["█ █", "█ █", "███", "  █", "  █"],
        '5' => ["███", "█  ", "███", "  █", "███"],
        '6' => ["███", "█  ", "███", "█ █", "███"],
        '7' => ["███", "  █", "  █", "  █", "  █"],
        '8' => ["███", "█ █", "███", "█ █", "███"],
        '9' => ["███", "█ █", "███", "  █", "███"],
        _ => [" ", "█", " ", "█", " "], // ':'
    }
}

/// Five rows of big text, every font pixel two cells wide.
fn big(text: &str) -> Vec<String> {
    (0..5)
        .map(|r| {
            let row = text
                .chars()
                .map(|c| glyph(c)[r])
                .collect::<Vec<_>>()
                .join("  ");
            row.chars().flat_map(|c| [c, c]).collect()
        })
        .collect()
}

pub struct Countdown {
    w: usize,
    h: usize,
    col: Rgb,
}

impl Default for Countdown {
    fn default() -> Self {
        Countdown {
            w: 0,
            h: 0,
            col: (255, 255, 255),
        }
    }
}

impl Animation for Countdown {
    fn resize(&mut self, w: usize, h: usize) {
        (self.w, self.h) = (w, h);
        self.col = hsv(Rng::new().f64(), 0.7, 1.0);
    }

    fn frame(&mut self, _dt: f64, clock: &Clock) -> Vec<String> {
        let (w, h) = (self.w, self.h);
        let text = fmt_clock(clock.remaining().map_or_else(|| clock.elapsed(), f64::ceil));
        let mut block = big(&text);
        if block[0].chars().count() > w || block.len() > h {
            block = vec![text];
        }
        let (bw, bh) = (block[0].chars().count(), block.len());
        let (bx, by) = (w.saturating_sub(bw) / 2, h.saturating_sub(bh) / 2);
        (0..h)
            .map(|y| match y.checked_sub(by).filter(|&i| i < bh) {
                Some(i) => {
                    let visible: String = block[i].chars().take(w - bx).collect();
                    format!(
                        "{}{}{visible}{}",
                        " ".repeat(bx),
                        fg(Some(self.col)),
                        " ".repeat(w.saturating_sub(bx + bw))
                    )
                }
                None => " ".repeat(w),
            })
            .collect()
    }
}
