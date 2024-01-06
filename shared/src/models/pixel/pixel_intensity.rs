use serde::{Serialize, Deserialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PixelIntensity {
    pub zn: f32,
    pub count: f32,
}
