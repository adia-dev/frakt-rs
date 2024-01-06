use crate::models::{
    fractal::fractal_descriptor::FractalDescriptor, range::Range, resolution::Resolution,
    u8_data::U8Data,
};

use serde::{Deserialize, Serialize};

use super::fragment::Fragment;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentTask {
    pub id: U8Data,
    pub fractal: FractalDescriptor,
    pub max_iteration: u32,
    pub resolution: Resolution,
    pub range: Range,
}

impl Fragment for FragmentTask {
    fn to_json(&self) -> Result<serde_json::Value, serde_json::Error> {
        let wrapped = serde_json::json!({ "FragmentTask": self });
        return serde_json::to_value(&wrapped);
    }

    fn from_json(fragment: &str) -> Result<Self, serde_json::Error> {
        let v: serde_json::Value = serde_json::from_str(fragment)?;
        serde_json::from_value(v["FragmentTask"].clone())
    }
}
