use crate::models::{
    pixel::pixel_data::PixelData, range::Range, resolution::Resolution, u8_data::U8Data,
};

use serde::{Deserialize, Serialize};

use super::fragment::Fragment;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentResult {
    pub id: U8Data,
    pub resolution: Resolution,
    pub range: Range,
    pub pixels: PixelData,
}

impl FragmentResult {
    pub fn new(id: U8Data, resolution: Resolution, range: Range, pixels: PixelData) -> Self {
        Self {
            id,
            resolution,
            range,
            pixels,
        }
    }
}

impl Fragment for FragmentResult {
    fn to_json(&self) -> Result<serde_json::Value, serde_json::Error> {
        let wrapped = serde_json::json!({ "FragmentResult": self });
        return serde_json::to_value(&wrapped);
    }

    fn from_json(fragment: &str) -> Result<Self, serde_json::Error> {
        let v: serde_json::Value = serde_json::from_str(fragment)?;
        serde_json::from_value(v["FragmentResult"].clone())
    }
}
