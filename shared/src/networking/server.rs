use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub address: String,
    pub port: u16,
    pub width: u16,
    pub height: u16,
}

impl Server {
    pub fn new(address: String, port: u16, width: u16, height: u16) -> Self {
        Self {
            address,
            port,
            width,
            height,
        }
    }
}
