use complex_rs::complex::Complex;
use serde::{Deserialize, Serialize};

use super::fractal::Fractal;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct NovaNewtonRaphsonZ3 {}

impl NovaNewtonRaphsonZ3 {
    pub fn new() -> Self {
        Self {}
    }

    fn fz(&self, z: Complex) -> Complex {
        z * z * z - Complex::new(1.0, 0.0)
    }

    fn dfz(&self, z: Complex) -> Complex {
        Complex::new(3.0, 0.0) * z * z
    }
}

impl Fractal for NovaNewtonRaphsonZ3 {
    fn generate(&self, max_iterations: u32, x: f64, y: f64) -> (f64, f64) {
        let c = Complex::new(x, y);
        let mut z = Complex::new(1.0, 0.0);
        let mut zn_next;
        let epsilon = 1e-6;
        let mut i = 0;

        loop {
            zn_next = z - (self.fz(z) / self.dfz(z)) + c;
            if (zn_next - z).arg_sq() < epsilon || i >= max_iterations {
                break;
            }
            z = zn_next;
            i += 1;
        }

        return (0.0, i as f64);
    }
}
