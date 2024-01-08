use complex_rs::complex::Complex;
use serde::{Deserialize, Serialize};

use super::fractal::Fractal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mandelbrot {}

impl Mandelbrot {
    pub fn new() -> Self {
        Self {}
    }
}

impl Fractal for Mandelbrot {
    fn generate(&self, max_iterations: u32, x: f64, y: f64) -> (f64, f64) {
        let mut z = Complex::new(0.0, 0.0);
        let c = Complex::new(x, y);

        let mut i = 0;
        while i < max_iterations && z.arg_sq() < 4.0 {
            z = z * z + c;
            i += 1;
        }

        return (z.arg_sq(), i as f64);
    }
}
