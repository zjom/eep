use std::f64::consts::TAU;

use super::{Animation, Clock};
use crate::color::{Rgb, encode_bg, hsv};

const TAB: usize = 1024;
const M: i64 = TAB as i64 - 1;
const K: f64 = TAB as f64 / TAU;

pub struct Plasma {
    w: usize,
    h: usize,
    t: f64,
    sin: Vec<f64>,
    rad: Vec<Vec<i64>>,
    pal: Vec<Rgb>,
}

impl Default for Plasma {
    fn default() -> Self {
        Plasma {
            w: 0,
            h: 0,
            t: 0.0,
            sin: (0..TAB)
                .map(|i| (TAU * i as f64 / TAB as f64).sin())
                .collect(),
            rad: vec![],
            pal: (0..256).map(|i| hsv(i as f64 / 256.0, 0.75, 1.0)).collect(),
        }
    }
}

impl Animation for Plasma {
    fn resize(&mut self, w: usize, h: usize) {
        (self.w, self.h) = (w, h);
        let (cx, cy) = (w as f64 / 2.0, h as f64); // pixel grid is w x 2h
        self.rad = (0..2 * h)
            .map(|y| {
                (0..w)
                    .map(|x| ((x as f64 - cx).hypot(y as f64 - cy) * 0.16 * K) as i64)
                    .collect()
            })
            .collect();
    }

    fn frame(&mut self, dt: f64, _clock: &Clock) -> Vec<String> {
        self.t += dt;
        let (t, w, h2, s) = (self.t, self.w, 2 * self.h, &self.sin);
        let at = |v: f64| s[((v * K) as i64 & M) as usize];
        let sx: Vec<f64> = (0..w).map(|x| at(x as f64 * 0.11 + t)).collect();
        let sy: Vec<f64> = (0..h2).map(|y| at(y as f64 * 0.15 - t * 1.3)).collect();
        let sd: Vec<f64> = (0..w + h2).map(|i| at(i as f64 * 0.07 + t * 0.7)).collect();
        let rt = (t * 1.9 * K) as i64;
        let shift = (t * 25.0) as i64;
        let px: Vec<Vec<Rgb>> = (0..h2)
            .map(|y| {
                let (yy, rr) = (sy[y], &self.rad[y]);
                (0..w)
                    .map(|x| {
                        let v = sx[x] + yy + sd[x + y] + s[((rr[x] - rt) & M) as usize] + 4.0;
                        self.pal[(((v * 32.0) as i64 + shift) & 255) as usize]
                    })
                    .collect()
            })
            .collect();
        let half = vec!['▀'; w];
        (0..self.h)
            .map(|y| encode_bg(&half, &px[2 * y], &px[2 * y + 1]))
            .collect()
    }
}
