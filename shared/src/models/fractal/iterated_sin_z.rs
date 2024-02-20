use complex_rs::complex::Complex;
use serde::{Deserialize, Serialize};

use super::fractal::Fractal;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct IteratedSinZ {
    pub c: Complex,
}

impl IteratedSinZ {
    pub fn new(c: Complex) -> Self {
        Self { c }
    }
}

impl Fractal for IteratedSinZ {
    fn generate(&self, max_iterations: u32, x: f64, y: f64) -> (f64, f64) {
        let mut z = Complex::new(x, y);

        let mut i = 0;
        while i < max_iterations && z.arg_sq() < 50.0 {
            z = z.sin() * self.c;
            i += 1;
        }

        return (z.arg_sq(), i as f64);
    }
}
