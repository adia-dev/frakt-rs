pub trait Fractal {
    fn generate(&self, max_iterations: u32, x: f64, y: f64) -> (f64, f64);
}
