use std::{collections::HashMap, net::SocketAddr};

use serde::{Deserialize, Serialize};

use crate::{
    models::{
        fractal::fractal_descriptor::FractalDescriptor, fragments::fragment_task::FragmentTask,
        range::Range,
    },
    networking::{
        server::{Server, ServerConfig},
        worker::Worker,
    },
};

// TODO: this struct is quite heavy to clone every time
// create smaller dtos focused on specific data, ServerTilesDto, ServerWorkersDto...
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
impl ServerDto {
    pub fn new(
        config: ServerConfig,
        tiles: Vec<Range>,
        tasks_queue: Vec<FragmentTask>,
        range: Range,
        current_fractal: usize,
        fractals: Vec<FractalDescriptor>,
        workers: HashMap<SocketAddr, Worker>,
    ) -> Self {
        Self {
            config,
            tiles,
            tasks_queue,
            range,
            current_fractal,
            fractals,
            workers,
        }
    }

    pub fn from_server(server: &Server) -> ServerDto {
        Self {
            config: server.config.clone(),
            tiles: server.tiles.clone(),
            tasks_queue: server.tasks_queue.clone(),
            range: server.range.clone(),
            current_fractal: server.current_fractal.clone(),
            fractals: server.fractals.clone(),
            workers: server.workers.clone(),
        }
    }
}
