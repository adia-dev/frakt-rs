use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct U8Data {
    pub offset: u32,
    pub count: u32,
}

impl U8Data {
    pub fn new(offset: u32, count: u32) -> Self {
        Self { offset, count }
    }
}
