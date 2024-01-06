use super::{julia::Julia, mandelbrot::Mandelbrot};

use serde::{Serialize, Deserialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FractalDescriptor {
    Julia(Julia),
    Mandelbrot(Mandelbrot)
}
