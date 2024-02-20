use complex_rs::complex::Complex;
use serde::{Deserialize, Serialize};

use super::fractal::Fractal;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct NewtonRaphsonZ3 {}

impl NewtonRaphsonZ3 {
    pub fn new() -> Self {
        Self {}
    }

    fn fz(&self, z: Complex) -> Complex {
        z * z * z - Complex::new(1.0, 0.0)
    }

    fn dfz(&self, z: Complex) -> Complex {
        Complex::new(3.0, 0.0) * z * z
    }

    fn convergence_value(pzn: f64, threshold: f64, count: u32, nmax: u32) -> f64 {
        let accuracy = f64::log10(threshold);
        if count < nmax {
            0.5 - 0.5 * f64::cos(0.1 * (count as f64 - (f64::log10(pzn) / accuracy)))
        } else {
            1.0
        }
    }
}

impl Fractal for NewtonRaphsonZ3 {
    fn generate(&self, max_iterations: u32, x: f64, y: f64) -> (f64, f64) {
        let mut z = Complex::new(x, y);
        let mut zn_next;
        let epsilon = 1e-6;
        let mut i = 0;

        loop {
            zn_next = z - (self.fz(z) / self.dfz(z));
            if (zn_next - z).arg_sq() < epsilon || i >= max_iterations {
                break;
            }
            z = zn_next;
            i += 1;
        }

        let zn = z.arg();
        let count = if i < max_iterations {
            NewtonRaphsonZ3::convergence_value(z.arg_sq(), epsilon, i, max_iterations)
        } else {
            1.0
        };

        return (zn, i as f64 * count);
    }
}
