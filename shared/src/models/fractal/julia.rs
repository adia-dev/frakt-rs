use complex_rs::complex::Complex;
use serde::{Deserialize, Serialize};

use super::fractal::Fractal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Julia {
    pub c: Complex,
    pub divergence_threshold_square: f64,
}

impl Julia {
    pub fn new(c: Complex, divergence_threshold_square: f64) -> Self {
        Self {
            c,
            divergence_threshold_square,
        }
    }
}

impl Fractal for Julia {
    fn generate(&self, max_iterations: u32, x: f64, y: f64) -> (f64, f64) {
        let mut z = Complex::new(x, y);

        let mut i = 0;
        while i < max_iterations && z.arg_sq() < self.divergence_threshold_square {
            z = z * z + self.c;
            i += 1;
        }

        return (z.arg_sq(), i as f64);
    }
}
