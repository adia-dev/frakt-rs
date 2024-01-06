use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PixelIntensity {
    pub zn: f32,
    pub count: f32,
}

impl PixelIntensity {
    pub fn new(zn: f32, count: f32) -> Self {
        Self { zn, count }
    }
}
