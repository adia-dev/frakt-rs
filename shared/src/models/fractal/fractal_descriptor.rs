use serde::{Deserialize, Serialize};

use super::{
    iterated_sin_z::IteratedSinZ, julia::Julia, mandelbrot::Mandelbrot,
    newton_raphson_3::NewtonRaphsonZ3, newton_raphson_4::NewtonRaphsonZ4,
    nova_newton_raphson_z3::NovaNewtonRaphsonZ3, nova_newton_raphson_z4::NovaNewtonRaphsonZ4,
};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FractalDescriptor {
    Julia(Julia),
    Mandelbrot(Mandelbrot),
    IteratedSinZ(IteratedSinZ),
    NewtonRaphsonZ3(NewtonRaphsonZ3),
    NewtonRaphsonZ4(NewtonRaphsonZ4),
    NovaNewtonRapshonZ3(NovaNewtonRaphsonZ3),
    NovaNewtonRapshonZ4(NovaNewtonRaphsonZ4),
}
