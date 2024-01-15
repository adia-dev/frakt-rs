use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub address: String,
    pub port: u16,
    pub width: u32,
    pub height: u32,
}

impl Server {
    pub fn new(address: String, port: u16, width: u32, height: u32) -> Self {
        Self {
            address,
            port,
            width,
            height,
        }
    }
}
