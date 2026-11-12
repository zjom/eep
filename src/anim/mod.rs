//! The animations, and the registry the CLI and driver use to find them.

mod clock;
mod fire;
mod life;
mod matrix;
mod pipes;
mod plasma;
mod stars;

use std::time::Instant;

pub struct Clock {
    total: Option<f64>,
    start: Instant,
}

impl Clock {
    pub fn new(total: Option<f64>) -> Self {
        Clock {
            total,
            start: Instant::now(),
        }
    }

    pub fn elapsed(&self) -> f64 {
        self.start.elapsed().as_secs_f64()
    }

    pub fn remaining(&self) -> Option<f64> {
        self.total.map(|t| (t - self.elapsed()).max(0.0))
    }
}

pub trait Animation {
    fn resize(&mut self, w: usize, h: usize);
    /// Advance by `dt` seconds; return exactly `h` lines of `w` visible cells.
    fn frame(&mut self, dt: f64, clock: &Clock) -> Vec<String>;
}

pub struct Info {
    pub name: &'static str,
    pub blurb: &'static str,
    pub make: fn(usize, usize) -> Box<dyn Animation>,
}

fn make<A: Animation + Default + 'static>(w: usize, h: usize) -> Box<dyn Animation> {
    let mut a = A::default();
    a.resize(w, h);
    Box::new(a)
}

pub const ALL: &[Info] = &[
    Info {
        name: "matrix",
        blurb: "falling green code rain",
        make: make::<matrix::Matrix>,
    },
    Info {
        name: "stars",
        blurb: "warp-speed starfield with a slow spin",
        make: make::<stars::Stars>,
    },
    Info {
        name: "fire",
        blurb: "a cosy crackling fire",
        make: make::<fire::Fire>,
    },
    Info {
        name: "plasma",
        blurb: "old-school demoscene plasma (half-block pixels)",
        make: make::<plasma::Plasma>,
    },
    Info {
        name: "life",
        blurb: "Conway's Game of Life, reseeds when it settles",
        make: make::<life::Life>,
    },
    Info {
        name: "pipes",
        blurb: "the classic pipes screensaver, in box-drawing characters",
        make: make::<pipes::Pipes>,
    },
    Info {
        name: "clock",
        blurb: "big countdown in the middle of the screen",
        make: make::<clock::Countdown>,
    },
];

pub fn index_of(name: &str) -> Option<usize> {
    ALL.iter().position(|a| a.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Visible cells in a line: our glyphs are all single-width, so count
    /// chars outside of CSI escape sequences.
    fn width(s: &str) -> usize {
        let mut n = 0;
        let mut chars = s.chars();
        while let Some(c) = chars.next() {
            if c == '\x1b' {
                chars.by_ref().find(|c| c.is_ascii_alphabetic());
            } else {
                n += 1;
            }
        }
        n
    }

    #[test]
    fn every_animation_fills_the_screen() {
        let clock = Clock::new(Some(125.0));
        for &(w, h) in &[(80, 24), (1, 1), (7, 3), (200, 60)] {
            for info in ALL {
                let mut a = (info.make)(w, h);
                for _ in 0..30 {
                    let lines = a.frame(0.05, &clock);
                    assert_eq!(lines.len(), h, "{} at {w}x{h}", info.name);
                    for l in &lines {
                        assert_eq!(width(l), w, "{} at {w}x{h}: {l:?}", info.name);
                    }
                }
                a.resize(w + 3, h + 2);
                assert_eq!(
                    a.frame(0.05, &clock).len(),
                    h + 2,
                    "{} after resize",
                    info.name
                );
            }
        }
    }
}
