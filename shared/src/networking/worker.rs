use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Worker {
    pub name: String,
    pub maximal_work_load: u32,
    pub address: String,
    pub port: u16,
}

impl Worker {
    pub fn new(name: String, maximal_work_load: u32, address: String, port: u16) -> Self {
        Self {
            name,
            maximal_work_load,
            address,
            port,
        }
    }
}
