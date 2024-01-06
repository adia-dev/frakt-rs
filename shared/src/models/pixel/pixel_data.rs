use serde::{Serialize, Deserialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PixelData {
    pub offset: u32,
    pub count: u32,
}
