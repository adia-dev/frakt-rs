use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PixelData {
    pub offset: u32,
    pub count: u32,
}

impl PixelData {
    pub fn new(offset: u32, count: u32) -> Self {
        Self { offset, count }
    }
}
