#![deny(clippy::all)]
#![forbid(unsafe_code)]

use std::sync::{Arc, Mutex};

use error_iter::ErrorIter as _;
use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use tokio::sync::mpsc::Receiver;
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

struct World {
    width: u32,
    height: u32,
    colors: Arc<Mutex<Vec<(u8, u8, u8)>>>,
}

pub async fn start_graphics(
    width: u32,
    height: u32,
    mut rx: Receiver<Vec<(u8, u8, u8)>>,
) -> Result<(), Error> {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();

    let shared_colors: Arc<Mutex<Vec<(u8, u8, u8)>>> = Arc::new(Mutex::new(Vec::new()));
    let shared_colors_clone = Arc::clone(&shared_colors);
    let mut world = World {
        width,
        height,
        colors: shared_colors,
    };
    tokio::spawn(async move {
        while let Some(data) = rx.recv().await {
            let mut colors = shared_colors_clone.lock().unwrap();
            *colors = data;
        }
    });

    let window = {
        let size = LogicalSize::new(world.width as f64, world.height as f64);
        WindowBuilder::new()
            .with_title("Frakt")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(world.width, world.height, surface_texture)?
    };

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            world.draw(pixels.frame_mut());
            if let Err(err) = pixels.render() {
                log_error("pixels.render", err);
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.close_requested() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    log_error("pixels.resize_surface", err);
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }

            // Update internal state and request a redraw
            world.update();
            window.request_redraw();
        }
    });
}

fn log_error<E: std::error::Error + 'static>(method_name: &str, err: E) {
    error!("{method_name}() failed: {err}");
    for source in err.sources().skip(1) {
        error!("  Caused by: {source}");
    }
}

impl World {
    fn update(&mut self) {}

    fn draw(&self, frame: &mut [u8]) {
        let colors = self.colors.lock().unwrap();

        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            if colors.is_empty() {
                let rgba = [0x0, 0x0, 0x0, 0xff];
                pixel.copy_from_slice(&rgba);
            } else {
                let (red, green, blue) = colors[i % colors.len()];
                let rgba = [red, green, blue, 0xff];
                pixel.copy_from_slice(&rgba);
            }
        }
    }
}
