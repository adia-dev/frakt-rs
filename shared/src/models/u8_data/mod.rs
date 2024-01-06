use serde::{Serialize, Deserialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct U8Data {
    pub offset: u32,
    pub count: u32,
}
