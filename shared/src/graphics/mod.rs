#![deny(clippy::all)]
#![forbid(unsafe_code)]

pub mod color;

use log::info;
use pixels::{Error, Pixels, SurfaceTexture};

use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::Receiver;
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

use crate::dtos::rendering_data::RenderingData;

use crate::models::range::Range;
use crate::networking::server::Server;

use self::color::PaletteHandler;

type SharedRenderingData = Arc<Vec<Mutex<Option<RenderingData>>>>;

struct GraphicsWorld {
    server: Arc<Mutex<Server>>,
    canvas_width: u32,
    canvas_height: u32,
    rendering_data_shards: SharedRenderingData,
    palette: PaletteHandler,
}

fn initialize_shared_data(shard_count: usize) -> SharedRenderingData {
    let shards = (0..shard_count).map(|_| Mutex::new(None)).collect();
    Arc::new(shards)
}

pub async fn launch_graphics_engine(
    server: Arc<Mutex<Server>>,
    mut rendering_data_receiver: Receiver<RenderingData>,
) -> Result<(), Error> {
    let event_loop = EventLoop::new();
    let mut input_helper = WinitInputHelper::new();

    let (canvas_width, canvas_height) = {
        let server = server.lock().unwrap();
        let width = server.config.width;
        let height = server.config.height;
        (width, height)
    };

    let rendering_data = initialize_shared_data(10);
    let mut graphics_world = GraphicsWorld {
        server,
        canvas_width,
        canvas_height,
        rendering_data_shards: rendering_data.clone(),
        palette: PaletteHandler::new(),
    };

    tokio::spawn(async move {
        loop {
            while let Some(data) = rendering_data_receiver.recv().await {
                for shard in rendering_data.iter() {
                    if let Ok(mut shard_lock) = shard.lock() {
                        if shard_lock.is_none() {
                            *shard_lock = Some(data.clone());
                            break;
                        }
                    }
                }
            }
        }
    });

    let window = {
        let size = LogicalSize::new(
            graphics_world.canvas_width as f64,
            graphics_world.canvas_height as f64,
        );
        WindowBuilder::new()
            .with_title("Frakt")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .expect("Failed to create window")
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(
            graphics_world.canvas_width,
            graphics_world.canvas_height,
            surface_texture,
        )?
    };

    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            graphics_world.render(pixels.frame_mut());
            if pixels.render().is_err() {
                *control_flow = ControlFlow::Exit;
            }
        }

        if input_helper.update(&event) {
            if input_helper.key_pressed(VirtualKeyCode::Escape) || input_helper.close_requested() {
                *control_flow = ControlFlow::Exit;
            }

            if input_helper.key_pressed(VirtualKeyCode::Right) {
                info!("Moving right");
                graphics_world.server.lock().unwrap().move_right();
            }
            if input_helper.key_pressed(VirtualKeyCode::Left) {
                info!("Moving left");
                graphics_world.server.lock().unwrap().move_left();
            }
            if input_helper.key_pressed(VirtualKeyCode::Down) {
                info!("Moving down");
                graphics_world.server.lock().unwrap().move_down();
            }
            if input_helper.key_pressed(VirtualKeyCode::Up) {
                info!("Moving up");
                graphics_world.server.lock().unwrap().move_up();
            }

            if input_helper.key_pressed(VirtualKeyCode::P) {
                graphics_world.server.lock().unwrap().zoom(0.9); // Zoom in
            }
            if input_helper.key_pressed(VirtualKeyCode::M) {
                graphics_world.server.lock().unwrap().zoom(1.1); // Zoom out
            }

            if input_helper.key_pressed(VirtualKeyCode::K) {
                graphics_world.server.lock().unwrap().cycle_fractal();
            }

            // TODO: C might be better to cycle through the fractal types
            if input_helper.key_pressed(VirtualKeyCode::C) {
                graphics_world.cycle_color_palette();
                window.request_redraw();
            }

            if let Some(size) = input_helper.window_resized() {
                pixels
                    .resize_surface(size.width, size.height)
                    .expect("Failed to resize surface");
            }

            graphics_world.update();
            window.request_redraw();
        }
    });
}

impl GraphicsWorld {
    fn update(&mut self) {}

    fn cycle_color_palette(&mut self) {
        self.palette.cycle_palette();
        self.server.lock().unwrap().regenerate_tiles();
    }

    fn render(&self, frame_buffer: &mut [u8]) {
        for shard in self.rendering_data_shards.iter() {
            if let Ok(mut data_lock) = shard.lock() {
                if let Some(render_data) = data_lock.take() {
                    // Safely take the value, replacing it with None
                    info!("Rendering result: {:?}", render_data.result);
                    let result = render_data.result;

                    let (start_x, start_y) = self.start_point(result.range);
                    info!("Drawing fragment at ({}, {})", start_x, start_y);

                    for y in 0..result.resolution.ny {
                        for x in 0..result.resolution.nx {
                            let t = render_data.iterations[(x + y * result.resolution.ny) as usize];
                            self.draw_pixel(
                                frame_buffer,
                                self.canvas_width,
                                start_x + x as u32,
                                start_y + y as u32,
                                t,
                            );
                        }
                    }
                }
            }
        }
    }

    // calculate the start point of the fragment, given the resolution and range and the server's range
    // the server range is the current view of the fractal, it is dynamic and changes as the user moves and zooms
    // we need to calculate the start point of the fragment in the canvas, given the resolution and the range of the fragment
    fn start_point(&self, range: Range) -> (u32, u32) {
        let server = self.server.lock().unwrap();
        let Range {
            min: server_min,
            max: server_max,
        } = server.range;
        let x = ((range.min.x - server_min.x) / (server_max.x - server_min.x)
            * self.canvas_width as f64) as u32;
        let y = ((range.min.y - server_min.y) / (server_max.y - server_min.y)
            * self.canvas_height as f64) as u32;

        (x, y)
    }

    fn draw_pixel(&self, frame_buffer: &mut [u8], canvas_width: u32, x: u32, y: u32, t: f64) {
        let index = ((y * canvas_width + x) * 4) as usize;

        if index + 3 < frame_buffer.len() {
            let (r, g, b) = self.palette.calculate_color(t);

            frame_buffer[index] = r;
            frame_buffer[index + 1] = g;
            frame_buffer[index + 2] = b;
            frame_buffer[index + 3] = 0xff;
        } else {
            panic!("Attempting to draw pixel outside the bounds of the frame buffer");
        }
    }
}
