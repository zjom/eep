use super::{Animation, Clock};
use crate::color::{Rgb, encode, gradient};
use crate::rng::Rng;

const N: usize = 48;
const STOPS: [(f64, Rgb); 6] = [
    (0.0, (0, 0, 0)),
    (0.12, (70, 5, 0)),
    (0.3, (190, 35, 0)),
    (0.5, (240, 110, 10)),
    (0.72, (255, 200, 50)),
    (1.0, (255, 255, 225)),
];

pub struct Fire {
    w: usize,
    h: usize,
    rng: Rng,
    heat: Vec<Vec<f64>>,
    cool: f64,
    pal: Vec<Rgb>,
    chars: Vec<char>,
}

impl Default for Fire {
    fn default() -> Self {
        let chars = (0..N)
            .map(|i| match i {
                0..3 => ' ',
                3..9 => '░',
                9..15 => '▒',
                15..22 => '▓',
                _ => '█',
            })
            .collect();
        Fire {
            w: 0,
            h: 0,
            rng: Rng::new(),
            heat: vec![],
            cool: 0.0,
            pal: gradient(&STOPS, N),
            chars,
        }
    }
}

impl Animation for Fire {
    fn resize(&mut self, w: usize, h: usize) {
        (self.w, self.h) = (w, h);
        self.heat = vec![vec![0.0; w]; h + 2];
        self.cool = 1.0 / (0.62 * h as f64).max(4.0);
    }

    fn frame(&mut self, _dt: f64, _clock: &Clock) -> Vec<String> {
        let (w, h) = (self.w, self.h);
        for y in [h, h + 1] {
            for v in &mut self.heat[y] {
                *v = self.rng.f64().powf(0.45);
            }
        }
        let k = self.cool;
        let wi = w as i64;
        for y in 0..h {
            let s = self.rng.choice(&[-1i64, 0, 0, 0, 1]); // a little sway
            let (below, below2) = (&self.heat[y + 1], &self.heat[y + 2]);
            let row: Vec<f64> = (0..w)
                .map(|i| {
                    let ii = i as i64;
                    let left = below[(ii - 1 + s).rem_euclid(wi) as usize];
                    let right = below[(ii + 1 + s).rem_euclid(wi) as usize];
                    let v = (left + below[i] + right + below2[i]) * 0.25 - k;
                    if v > 0.0 { v } else { 0.0 }
                })
                .collect();
            self.heat[y] = row;
        }
        let top = N - 1;
        self.heat[..h]
            .iter()
            .map(|row| {
                let idx: Vec<usize> = row
                    .iter()
                    .map(|&v| ((v * top as f64) as usize).min(top))
                    .collect();
                let chars: Vec<char> = idx.iter().map(|&i| self.chars[i]).collect();
                let fgs: Vec<Option<Rgb>> = idx.iter().map(|&i| Some(self.pal[i])).collect();
                encode(&chars, &fgs)
            })
            .collect()
    }
}
