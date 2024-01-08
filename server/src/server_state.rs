use std::collections::HashMap;

use shared::networking::worker::Worker;

#[derive(Debug, Clone)]
pub struct ServerState {
    pub metrics: HashMap<String, u64>,
    pub workers: HashMap<String, Worker>,
}

impl ServerState {
    pub fn new() -> Self {
        ServerState {
            metrics: HashMap::new(),
            workers: HashMap::new(),
        }
    }
}
