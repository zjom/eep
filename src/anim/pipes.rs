use super::{Animation, Clock};
use crate::color::{Rgb, encode};
use crate::rng::Rng;

const COLORS: [Rgb; 7] = [
    (255, 95, 95),
    (255, 175, 0),
    (255, 235, 95),
    (95, 255, 135),
    (95, 215, 255),
    (175, 135, 255),
    (255, 135, 215),
];
// Directions: up, right, down, left.
const DX: [i64; 4] = [0, 1, 0, -1];
const DY: [i64; 4] = [-1, 0, 1, 0];
const SPEED: f64 = 30.0; // cells per second, per pipe

/// The box-drawing piece joining two sides of a cell.
fn piece(a: usize, b: usize) -> char {
    match (a.min(b), a.max(b)) {
        (0, 2) => '│',
        (1, 3) => '─',
        (1, 2) => '╭',
        (2, 3) => '╮',
        (0, 1) => '╰',
        _ => '╯', // (0, 3)
    }
}

struct Pipe {
    x: usize,
    y: usize,
    dir: usize,
    col: Rgb,
}

pub struct Pipes {
    w: usize,
    h: usize,
    rng: Rng,
    chars: Vec<Vec<char>>,
    cols: Vec<Vec<Option<Rgb>>>,
    drawn: usize,
    pipes: Vec<Pipe>,
    acc: f64,
}

impl Default for Pipes {
    fn default() -> Self {
        Pipes {
            w: 0,
            h: 0,
            rng: Rng::new(),
            chars: vec![],
            cols: vec![],
            drawn: 0,
            pipes: vec![],
            acc: 0.0,
        }
    }
}

impl Pipes {
    fn clear(&mut self) {
        self.chars = vec![vec![' '; self.w]; self.h];
        self.cols = vec![vec![None; self.w]; self.h];
        self.drawn = 0;
    }

    fn new_pipe(&mut self) -> Pipe {
        Pipe {
            x: self.rng.below(self.w),
            y: self.rng.below(self.h),
            dir: self.rng.below(4),
            col: self.rng.choice(&COLORS),
        }
    }

    fn advance(&mut self, i: usize) {
        let Pipe { x, y, dir, col } = self.pipes[i];
        let nd = if self.rng.f64() < 0.13 {
            (dir + self.rng.choice(&[1, 3])) % 4
        } else {
            dir
        };
        self.chars[y][x] = piece((dir + 2) % 4, nd);
        self.cols[y][x] = Some(col);
        let p = &mut self.pipes[i];
        p.x = (x as i64 + DX[nd]).rem_euclid(self.w as i64) as usize;
        p.y = (y as i64 + DY[nd]).rem_euclid(self.h as i64) as usize;
        p.dir = nd;
        self.drawn += 1;
        if self.rng.f64() < 0.004 {
            self.pipes[i] = self.new_pipe();
        }
    }
}

impl Animation for Pipes {
    fn resize(&mut self, w: usize, h: usize) {
        (self.w, self.h) = (w, h);
        self.clear();
        self.pipes = (0..4).map(|_| self.new_pipe()).collect();
        self.acc = 0.0;
    }

    fn frame(&mut self, dt: f64, _clock: &Clock) -> Vec<String> {
        self.acc += dt * SPEED;
        let steps = self.acc as usize;
        self.acc %= 1.0;
        for _ in 0..steps {
            for i in 0..self.pipes.len() {
                self.advance(i);
            }
        }
        if self.drawn as f64 > (self.w * self.h) as f64 * 1.6 {
            self.clear();
        }
        self.chars
            .iter()
            .zip(&self.cols)
            .map(|(c, f)| encode(c, f))
            .collect()
    }
}
