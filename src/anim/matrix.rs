use super::{Animation, Clock};
use crate::color::{Rgb, encode};
use crate::rng::Rng;

const GLYPHS: &str = "ｱｲｳｴｵｶｷｸｹｺｻｼｽｾｿﾀﾁﾂﾃﾄﾅﾆﾇﾈﾉﾊﾋﾌﾍﾎﾏﾐﾑﾒﾓﾔﾕﾖﾗﾘﾙﾚﾛﾜﾝ0123456789:.=*+-<>|";
const LEVELS: usize = 24;
const HEAD: Rgb = (225, 255, 225);

/// A falling drop: head y, speed (cells/s), fade rate (life/s).
struct Drop {
    y: f64,
    speed: f64,
    fade: f64,
}

pub struct Matrix {
    w: usize,
    h: usize,
    rng: Rng,
    glyphs: Vec<char>,
    chars: Vec<Vec<char>>,
    life: Vec<Vec<f64>>,
    drops: Vec<Drop>,
    pal: Vec<Rgb>,
}

impl Default for Matrix {
    fn default() -> Self {
        let pal = (0..LEVELS)
            .map(|i| {
                let l = i as f64 / (LEVELS - 1) as f64;
                (
                    (10.0 + 60.0 * l.powi(3)) as u8,
                    (35.0 + 220.0 * l) as u8,
                    (15.0 + 80.0 * l * l) as u8,
                )
            })
            .collect();
        Matrix {
            w: 0,
            h: 0,
            rng: Rng::new(),
            glyphs: GLYPHS.chars().collect(),
            chars: vec![],
            life: vec![],
            drops: vec![],
            pal,
        }
    }
}

impl Matrix {
    fn new_drop(&mut self, initial: bool) -> Drop {
        let reach = self.h as f64 * if initial { 1.5 } else { 0.6 };
        Drop {
            y: -self.rng.uniform(0.0, reach),
            speed: self.rng.uniform(6.0, 30.0),
            fade: self.rng.uniform(0.35, 1.3),
        }
    }
}

impl Animation for Matrix {
    fn resize(&mut self, w: usize, h: usize) {
        (self.w, self.h) = (w, h);
        self.chars = vec![vec![' '; w]; h];
        self.life = vec![vec![0.0; w]; h];
        self.drops = (0..w).map(|_| self.new_drop(true)).collect();
    }

    fn frame(&mut self, dt: f64, _clock: &Clock) -> Vec<String> {
        let (w, h) = (self.w, self.h);
        for row in &mut self.life {
            for (v, d) in row.iter_mut().zip(&self.drops) {
                let dec = d.fade * dt;
                *v = if *v > dec { *v - dec } else { 0.0 };
            }
        }

        for x in 0..w {
            let y0 = self.drops[x].y as i64;
            let d = &mut self.drops[x];
            d.y += d.speed * dt;
            let end = (d.y as i64).min(h as i64 - 1);
            for y in (y0 + 1).max(0)..=end {
                self.chars[y as usize][x] = self.rng.choice(&self.glyphs);
                self.life[y as usize][x] = 1.0;
            }
            let d = &self.drops[x];
            if d.y - d.speed / d.fade > h as f64 {
                // whole trail has left the screen
                self.drops[x] = self.new_drop(false);
            }
        }

        for _ in 0..w * h / 150 {
            // glitchy flicker in the trails
            let (y, x) = (self.rng.below(h), self.rng.below(w));
            if self.life[y][x] > 0.15 {
                self.chars[y][x] = self.rng.choice(&self.glyphs);
            }
        }

        let top = (LEVELS - 1) as f64;
        (0..h)
            .map(|y| {
                let (lr, cr) = (&self.life[y], &self.chars[y]);
                let chars: Vec<char> = cr
                    .iter()
                    .zip(lr)
                    .map(|(&c, &l)| if l > 0.02 { c } else { ' ' })
                    .collect();
                let mut fgs: Vec<Option<Rgb>> = lr
                    .iter()
                    .map(|&l| Some(self.pal[(l * top) as usize]))
                    .collect();
                for (x, d) in self.drops.iter().enumerate() {
                    if d.y as i64 == y as i64 {
                        fgs[x] = Some(HEAD);
                    }
                }
                encode(&chars, &fgs)
            })
            .collect()
    }
}
