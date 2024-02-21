use std::{collections::HashMap, net::SocketAddr};

use complex_rs::complex::Complex;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Sender;

use crate::{
    dtos::rendering_data::RenderingData,
    models::{
        fractal::{
            fractal_descriptor::FractalDescriptor, iterated_sin_z::IteratedSinZ, julia::Julia,
            mandelbrot::Mandelbrot, newton_raphson_3::NewtonRaphsonZ3,
            newton_raphson_4::NewtonRaphsonZ4, nova_newton_raphson_z3::NovaNewtonRaphsonZ3,
            nova_newton_raphson_z4::NovaNewtonRaphsonZ4,
        },
        fragments::fragment_task::FragmentTask,
        point::Point,
        range::Range,
        resolution::Resolution,
        u8_data::U8Data,
    },
};

use super::worker::Worker;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub address: String,
    pub port: u16,
    pub width: u32,
    pub height: u32,
    pub tiles: u32,
    pub range: Range,
    pub speed: f64,
}

impl ServerConfig {
    pub fn new(address: String, port: u16, width: u32, height: u32, tiles: u32) -> Self {
        let min = Point::new(-1.2, -1.2);
        let max = Point::new(1.2, 1.2);
        let range = Range::new(min, max);
        let speed = 1.0;

        Self {
            address,
            port,
            width,
            height,
            tiles,
            range,
            speed,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Server {
    pub config: ServerConfig,
    pub render_tx: Sender<RenderingData>,
    pub tiles: Vec<Range>,
    pub range: Range,
    pub current_fractal: usize,
    pub fractals: Vec<FractalDescriptor>,
    pub workers: HashMap<SocketAddr, Worker>,
}

impl Server {
    pub fn new(config: ServerConfig, render_tx: Sender<RenderingData>) -> Self {
        let range = config.range;
        let workers: HashMap<SocketAddr, Worker> = HashMap::new();
        let tiles = Server::generate_tiles(&range, config.tiles);
        let fractals: Vec<FractalDescriptor> = vec![
            FractalDescriptor::Mandelbrot(Mandelbrot::new()),
            FractalDescriptor::Julia(Julia::new(
                complex_rs::complex::Complex {
                    re: 0.285,
                    im: 0.013,
                },
                2.0,
            )),
            FractalDescriptor::Julia(Julia::new(
                complex_rs::complex::Complex {
                    re: -0.9,
                    im: 0.276015,
                },
                2.0,
            )),
            FractalDescriptor::IteratedSinZ(IteratedSinZ::new(Complex { re: 1.0, im: 0.3 })),
            FractalDescriptor::IteratedSinZ(IteratedSinZ::new(Complex { re: 0.2, im: 1.0 })),
            FractalDescriptor::NewtonRaphsonZ3(NewtonRaphsonZ3::new()),
            FractalDescriptor::NewtonRaphsonZ4(NewtonRaphsonZ4::new()),
            FractalDescriptor::NovaNewtonRapshonZ3(NovaNewtonRaphsonZ3::new()),
            FractalDescriptor::NovaNewtonRapshonZ4(NovaNewtonRaphsonZ4::new()),
        ];

        Self {
            config,
            render_tx,
            tiles,
            range,
            current_fractal: 0,
            fractals,
            workers,
        }
    }

    pub fn cycle_fractal(&mut self) {
        self.current_fractal = (self.current_fractal + 1) % self.fractals.len();
        self.regenerate_tiles();
    }

    pub fn register_worker(&mut self, addr: SocketAddr, worker: Worker) {
        self.workers.insert(addr, worker);
    }

    pub fn get_worker(&self, addr: &SocketAddr) -> Option<&Worker> {
        self.workers.get(addr)
    }

    pub fn create_fragment_task(&mut self) -> Option<FragmentTask> {
        let config = self.config.clone();

        if let Some(range) = self.get_random_tile() {
            let id = U8Data::new(0, 16);
            let fractal_descriptor = self.fractals[self.current_fractal].clone();
            // TODO: the max iterations should change based on the current fractal
            let max_iterations = 256;
            let resolution = self.calculate_resolution(config.width, config.height, config.tiles);
            let range = range;

            Some(FragmentTask::new(
                id,
                fractal_descriptor,
                max_iterations,
                resolution,
                range,
            ))
        } else {
            None
        }
    }

    pub fn regenerate_tiles(&mut self) {
        self.tiles = Server::generate_tiles(&self.range, self.config.tiles);
    }

    pub fn move_right(&mut self) {
        self._move(self.config.speed, 0.0);
    }

    pub fn move_left(&mut self) {
        self._move(-self.config.speed, 0.0);
    }

    pub fn move_up(&mut self) {
        self._move(0.0, self.config.speed);
    }

    pub fn move_down(&mut self) {
        self._move(0.0, -self.config.speed);
    }

    fn _move(&mut self, x: f64, y: f64) {
        let tile_width = (self.range.max.x - self.range.min.x) / self.config.tiles as f64;
        let tile_height = (self.range.max.y - self.range.min.y) / self.config.tiles as f64;

        let dx = tile_width * x;
        let dy = tile_height * y;

        self.range.min.x += dx;
        self.range.max.x += dx;
        self.range.min.y += dy;
        self.range.max.y += dy;

        self.regenerate_tiles();
    }

    pub fn zoom(&mut self, factor: f64) {
        let tile_width = (self.range.max.x - self.range.min.x) / self.config.tiles as f64;
        let tile_height = (self.range.max.y - self.range.min.y) / self.config.tiles as f64;

        let dx = tile_width * factor;
        let dy = tile_height * factor;

        self.range.min.x += dx;
        self.range.max.x -= dx;
        self.range.min.y += dy;
        self.range.max.y -= dy;

        self.regenerate_tiles();
    }

    pub fn get_random_tile(&mut self) -> Option<Range> {
        if self.tiles.is_empty() {
            None
        } else {
            let mut rng = thread_rng();
            let len = self.tiles.len();
            self.tiles.remove(rng.gen_range(0..len)).into()
        }
    }

    pub fn calculate_resolution(&self, width: u32, height: u32, tiles: u32) -> Resolution {
        let tile_width = (width / tiles) as u16;
        let tile_height = (height / tiles) as u16;

        Resolution::new(tile_width, tile_height)
    }

    pub fn calculate_range(id: u8, tiles: u32, range: &Range) -> Range {
        let tile_width = (range.max.x - range.min.x) / tiles as f64;
        let tile_height = (range.max.y - range.min.y) / tiles as f64;

        let x = (id % tiles as u8) as f64;
        let y = (id / tiles as u8) as f64;

        let min = Point::new(range.min.x + x * tile_width, range.min.y + y * tile_height);
        let max = Point::new(
            range.min.x + (x + 1.0) * tile_width,
            range.min.y + (y + 1.0) * tile_height,
        );

        Range::new(min, max)
    }

    fn generate_tiles(range: &Range, count: u32) -> Vec<Range> {
        let mut ranges = Vec::new();
        for i in 0..(count * count) {
            let range = Server::calculate_range(i as u8, count, range);
            ranges.push(range);
        }

        ranges
    }
}
