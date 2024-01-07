use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub address: String,
    pub port: u16,
}

impl Server {
    pub fn new(address: String, port: u16) -> Self {
        Self { address, port }
    }
}
