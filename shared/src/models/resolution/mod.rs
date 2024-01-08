use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Resolution {
    pub nx: u16,
    pub ny: u16,
}

impl Resolution {
    pub fn new(nx: u16, ny: u16) -> Self {
        Self { nx, ny }
    }
}
