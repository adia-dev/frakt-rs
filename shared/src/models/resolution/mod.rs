use serde::{Serialize, Deserialize};
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Resolution {
    pub nx: u16,
    pub ny: u16,
}
