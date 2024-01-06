use serde::{Deserialize, Serialize};

use super::fragment::Fragment;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentRequest {
    pub worker_name: String,
    pub maximal_work_load: u32,
}

impl FragmentRequest {
    pub fn new(worker_name: String, maximum_work_load: u32) -> Self {
        Self {
            worker_name,
            maximal_work_load: maximum_work_load,
        }
    }
}

impl Fragment for FragmentRequest {
    fn to_json(&self) -> Result<serde_json::Value, serde_json::Error> {
        let wrapped = serde_json::json!({ "FragmentRequest": self });
        return serde_json::to_value(&wrapped);
    }

    fn from_json(fragment: &str) -> Result<Self, serde_json::Error> {
        let v: serde_json::Value = serde_json::from_str(fragment)?;
        serde_json::from_value(v["FragmentRequest"].clone())
    }
}
