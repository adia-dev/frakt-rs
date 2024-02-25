use std::{collections::HashMap, net::SocketAddr};

use serde::{Deserialize, Serialize};

use crate::{models::{fractal::fractal_descriptor::FractalDescriptor, fragments::fragment_task::FragmentTask, range::Range}, networking::{server::ServerConfig, worker::Worker}};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerDto {
    pub config: ServerConfig,
    pub tiles: Vec<Range>,
    pub tasks_queue: Vec<FragmentTask>,
    pub range: Range,
    pub current_fractal: usize,
    pub fractals: Vec<FractalDescriptor>,
    pub workers: HashMap<SocketAddr, Worker>,
}
