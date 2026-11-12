use super::{Animation, Clock};
use crate::color::{Rgb, encode};
use crate::rng::Rng;

const CH: [char; 5] = ['.', '·', '•', '*', '✦'];
const LEVELS: usize = 16;

pub struct Stars {
    w: usize,
    h: usize,
    rng: Rng,
    stars: Vec<[f64; 3]>,
    pal: Vec<Rgb>,
}

impl Default for Stars {
    fn default() -> Self {
        let pal = (0..LEVELS)
            .map(|i| {
                let l = i as f64 / (LEVELS - 1) as f64;
                (
                    (90.0 + 165.0 * l) as u8,
                    (100.0 + 155.0 * l) as u8,
                    (140.0 + 115.0 * l) as u8,
                )
            })
            .collect();
        Stars {
            w: 0,
            h: 0,
            rng: Rng::new(),
            stars: vec![],
            pal,
        }
    }
}

impl Stars {
    fn star(&mut self, z: f64) -> [f64; 3] {
        [self.rng.uniform(-1.0, 1.0), self.rng.uniform(-1.0, 1.0), z]
    }
}

impl Animation for Stars {
    fn resize(&mut self, w: usize, h: usize) {
        (self.w, self.h) = (w, h);
        let n = (w * h / 16).max(40);
        self.stars = (0..n)
            .map(|_| {
                let z = self.rng.uniform(0.05, 1.0);
                self.star(z)
            })
            .collect();
    }

    fn frame(&mut self, dt: f64, _clock: &Clock) -> Vec<String> {
        let (w, h) = (self.w, self.h);
        let (cx, cy) = (w as f64 / 2.0, h as f64 / 2.0);
        let (sa, ca) = (0.08 * dt).sin_cos();
        let mut chars = vec![vec![' '; w]; h];
        let mut fgs: Vec<Vec<Option<Rgb>>> = vec![vec![None; w]; h];
        let mut best = vec![vec![-1.0f64; w]; h];
        for i in 0..self.stars.len() {
            let s = &mut self.stars[i];
            s[2] -= 0.28 * dt;
            (s[0], s[1]) = (s[0] * ca - s[1] * sa, s[0] * sa + s[1] * ca);
            if s[2] <= 0.02 {
                self.stars[i] = self.star(1.0);
                continue;
            }
            let sx = (cx + s[0] / s[2] * w as f64 * 0.22) as i64;
            let sy = (cy + s[1] / s[2] * h as f64 * 0.22) as i64;
            if !(0..w as i64).contains(&sx) || !(0..h as i64).contains(&sy) {
                self.stars[i] = self.star(1.0);
                continue;
            }
            let (sx, sy) = (sx as usize, sy as usize);
            let b = 1.0 - s[2];
            if best[sy][sx] < b {
                best[sy][sx] = b;
                chars[sy][sx] = CH[((b * b * 5.5) as usize).min(4)];
                fgs[sy][sx] = Some(self.pal[(b * (LEVELS - 1) as f64) as usize]);
            }
        }
        chars.iter().zip(&fgs).map(|(c, f)| encode(c, f)).collect()
    }
}
