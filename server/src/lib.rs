use std::mem::size_of;

use complex_rs::complex::Complex;
use image::{ImageBuffer, Rgb};
use log::{debug, error, info, trace};
use shared::{
    graphics::start_graphics,
    models::{
        fractal::{fractal_descriptor::FractalDescriptor, julia::Julia, mandelbrot::Mandelbrot},
        fragments::{
            fragment::Fragment, fragment_request::FragmentRequest, fragment_result::FragmentResult,
            fragment_task::FragmentTask,
        },
        pixel::pixel_intensity::PixelIntensity,
        point::Point,
        range::Range,
        resolution::Resolution,
        u8_data::U8Data,
    },
    networking::{read_message_raw, result::NetworkingResult, send_message, server::Server},
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{
        broadcast,
        mpsc::{self, Sender},
    },
};

pub async fn run_server(server: &Server) {
    match run(&server).await {
        Ok(()) => info!("Server shutdown gracefully"),
        Err(e) => error!("Server error: {}", e),
    }
}

async fn run(server: &Server) -> NetworkingResult<()> {
    let server_addr = format!("{}:{}", server.address, server.port);
    let listener = start_server(&server_addr).await?;
    info!("Server listening on {}", server_addr);

    let (rendering_tx, rendering_rx) = mpsc::channel::<Vec<(u8, u8, u8)>>(32);
    info!("Launched the rendering channels !");

    // Spawn a task to handle connections
    let server_handle = {
        tokio::spawn(async move {
            while let Ok((mut socket, _)) = listener.accept().await {
                let rendering_tx = rendering_tx.clone();

                tokio::spawn(async move {
                    tokio::select! {
                        result = handle_connection(&mut socket, rendering_tx) => {
                            if let Err(e) = result {
                                error!("Connection handling error: {:?}", e);
                            }
                        },
                    }
                });
            }
        })
    };

    let graphics_handle = start_graphics(server.width, server.height, rendering_rx);

    let _ = tokio::join!(server_handle, graphics_handle);

    Ok(())
}

async fn start_server(addr: &str) -> NetworkingResult<TcpListener> {
    Ok(TcpListener::bind(addr).await?)
}

async fn handle_connection(
    mut socket: &mut TcpStream,
    rendering_tx: Sender<Vec<(u8, u8, u8)>>,
) -> NetworkingResult<()> {
    debug!("Handling new connection...");
    let raw_message = read_message_raw(&mut socket).await?;
    trace!("{:?}", raw_message);

    if let Ok(result) = FragmentResult::from_json(&raw_message.json_message) {
        info!("Received a FragmentResult !");
        debug!("{:?}", result);
        _ = tokio::spawn(async move {
            // Read the first 4 bytes that contains the signature
            // NOTE: I've written 4 but it will eventually become dynamic
            let (signature_bytes, pixel_data) = raw_message.data.split_at(16);
            info!("Task Signature received and verified !");
            // TODO: implement the actual verification of the signature
            debug!("Task Signature: {:02X?}", signature_bytes);

            // Now process the rest as pixel intensities
            let pixel_intensities: Vec<PixelIntensity> = pixel_data
                .chunks_exact(size_of::<PixelIntensity>())
                .map(|pixel_intensity_chunk| {
                    let (zn_bytes, count_bytes) = pixel_intensity_chunk.split_at(4);
                    // NOTE: Do I really need to create a PixelIntensity instance ? a tuple is
                    // enough right ?
                    PixelIntensity {
                        zn: f32::from_be_bytes(zn_bytes.try_into().unwrap()),
                        count: f32::from_be_bytes(count_bytes.try_into().unwrap()),
                    }
                })
                .collect();

            let mut image_buffer: ImageBuffer<Rgb<u8>, Vec<u8>> =
                ImageBuffer::new(result.resolution.nx as u32, result.resolution.ny as u32);

            let mut colors: Vec<(u8, u8, u8)> = Vec::new();
            for (x, y, _pixel) in image_buffer.enumerate_pixels_mut() {
                let (i, zn) = (
                    pixel_intensities[(y * (result.resolution.nx as u32) + x) as usize].count,
                    pixel_intensities[(y * (result.resolution.nx as u32) + x) as usize].zn,
                );
                let t = (i as f32 - zn.log2().log2()) / (64 as f32);

                let [red, green, blue] = color((40.0 * t + 0.5) % 1.0);
                colors.push((red, green, blue));
            }

            _ = rendering_tx.send(colors).await;
        })
        .await;
        return Ok(());
    };

    let request = FragmentRequest::from_json(&raw_message.json_message)?;
    info!("FragmentRequest received successfully !");
    debug!("{:?}", request);

    send_fragment_task(socket, &request.worker_name).await?;

    Ok(())
}

fn color(t: f32) -> [u8; 3] {
    let a = (0.910, 0.541, 0.988);
    let b = (0.927, 0.211, 0.790);
    let c = (1.285, 1.294, 0.802);
    let d = (2.910, 4.973, 1.429);

    let r = b.0 * (6.28318 * (c.0 * t + d.0)).cos() + a.0;
    let g = b.1 * (6.28318 * (c.1 * t + d.1)).cos() + a.1;
    let b = b.2 * (6.28318 * (c.2 * t + d.2)).cos() + a.2;
    [(255.0 * r) as u8, (255.0 * g) as u8, (255.0 * b) as u8]
}

async fn send_fragment_task(stream: &mut TcpStream, worker_name: &str) -> NetworkingResult<()> {
    let id = U8Data::new(0, 16);

    let mandelbrot = Mandelbrot::new();
    let fractal_descriptor = FractalDescriptor::Mandelbrot(mandelbrot);
    let max_iterations: u32 = 64;

    let resolution = Resolution::new(500, 500);
    let range = Range::new(Point::new(-1.2, -1.2), Point::new(1.2, 1.2));

    let task = FragmentTask::new(id, fractal_descriptor, max_iterations, resolution, range);
    let task_json = task.to_json()?;

    // TODO: Randomize the signature for each task sent
    let signature = [0u8; 16];

    let serialized_fragment_task = serde_json::to_string(&task_json)?;
    let serialized_fragment_task_bytes = serialized_fragment_task.as_bytes();

    if let Err(e) = send_message(stream, serialized_fragment_task_bytes, Some(&signature)).await {
        error!("Failed to send task: {}", e);
        return Err(e.into());
    }
    info!("FragmentTask sent to the worker {} !", worker_name);
    debug!("{:?}", task);

    Ok(())
}
