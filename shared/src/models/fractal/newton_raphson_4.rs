use complex_rs::complex::Complex;
use serde::{Deserialize, Serialize};

use super::{fractal::Fractal, utils};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct NewtonRaphsonZ4 {}

impl NewtonRaphsonZ4 {
    pub fn new() -> Self {
        Self {}
    }

    fn fz(&self, z: Complex) -> Complex {
        z * z * z * z - Complex::new(1.0, 0.0)
    }

    fn dfz(&self, z: Complex) -> Complex {
        Complex::new(4.0, 0.0) * z * z * z
    }
}

impl Fractal for NewtonRaphsonZ4 {
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
            utils::convergence_value(z.arg_sq(), epsilon, i, max_iterations)
        } else {
            1.0
        };

        return (zn, i as f64 * count);
    }
}
