use std::collections::VecDeque;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use super::{Animation, Clock};
use crate::color::{fg, hsv};
use crate::rng::Rng;

const GPS: f64 = 14.0; // generations per second

/// Toroidal grid of `w` x `2h` cells, drawn two cells per character.
pub struct Life {
    w: usize,
    h: usize,
    gw: usize,
    gh: usize,
    t: f64,
    acc: f64,
    rng: Rng,
    cells: Vec<bool>,
    seen: VecDeque<u64>,
    stale: f64,
}

impl Default for Life {
    fn default() -> Self {
        Life {
            w: 0,
            h: 0,
            gw: 0,
            gh: 0,
            t: 0.0,
            acc: 0.0,
            rng: Rng::new(),
            cells: vec![],
            seen: VecDeque::new(),
            stale: 0.0,
        }
    }
}

impl Life {
    fn seed(&mut self) {
        // P(alive) = 11/32, same density as the bitwise seed in the Python version.
        self.cells = (0..self.gw * self.gh)
            .map(|_| self.rng.f64() < 11.0 / 32.0)
            .collect();
        self.seen.clear();
        self.stale = 0.0;
    }

    fn step(&mut self) {
        let (gw, gh) = (self.gw, self.gh);
        let old = &self.cells;
        let mut new = vec![false; old.len()];
        for y in 0..gh {
            let rows = [(y + gh - 1) % gh, y, (y + 1) % gh];
            for x in 0..gw {
                let cols = [(x + gw - 1) % gw, x, (x + 1) % gw];
                let mut n = 0;
                for (i, &ry) in rows.iter().enumerate() {
                    for (j, &cx) in cols.iter().enumerate() {
                        if (i, j) != (1, 1) && old[ry * gw + cx] {
                            n += 1;
                        }
                    }
                }
                new[y * gw + x] = n == 3 || (n == 2 && old[y * gw + x]);
            }
        }
        self.cells = new;
    }
}

impl Animation for Life {
    fn resize(&mut self, w: usize, h: usize) {
        (self.w, self.h) = (w, h);
        (self.gw, self.gh) = (w.max(2), (2 * h).max(2));
        self.acc = 0.0;
        self.seed();
    }

    fn frame(&mut self, dt: f64, _clock: &Clock) -> Vec<String> {
        self.t += dt;
        self.acc += dt * GPS;
        while self.acc >= 1.0 {
            self.acc -= 1.0;
            self.step();
            let mut hasher = DefaultHasher::new();
            self.cells.hash(&mut hasher);
            let key = hasher.finish();
            if self.seen.contains(&key) {
                self.stale += 1.0 / GPS;
            }
            self.seen.push_back(key);
            if self.seen.len() > 40 {
                self.seen.pop_front();
            }
        }
        if self.stale > 2.5 || !self.cells.contains(&true) {
            self.seed();
        }

        let gw = self.gw;
        (0..self.h)
            .map(|y| {
                let col = hsv(
                    0.55 + 0.25 * y as f64 / self.h.max(1) as f64 + self.t * 0.03,
                    0.55,
                    1.0,
                );
                let mut line = fg(Some(col));
                for x in 0..self.w {
                    let top = self.cells[2 * y * gw + x];
                    let bot = self.cells[(2 * y + 1) * gw + x];
                    line.push(match (top, bot) {
                        (false, false) => ' ',
                        (true, false) => '▀',
                        (false, true) => '▄',
                        (true, true) => '█',
                    });
                }
                line
            })
            .collect()
    }
}
