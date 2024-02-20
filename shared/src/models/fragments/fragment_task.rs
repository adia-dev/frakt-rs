use crate::models::{
    fractal::{fractal::Fractal, fractal_descriptor::FractalDescriptor},
    pixel::{pixel_data::PixelData, pixel_intensity::PixelIntensity},
    range::Range,
    resolution::Resolution,
    u8_data::U8Data,
};
use image::{ImageBuffer, Rgb};
use log::{debug, error};
use serde::{Deserialize, Serialize};
use std::io::Write;

use super::{fragment::Fragment, fragment_result::FragmentResult};

type FragmentResultData = Vec<u8>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentTask {
    pub id: U8Data,
    pub fractal: FractalDescriptor,
    pub max_iteration: u32,
    pub resolution: Resolution,
    pub range: Range,
}

impl FragmentTask {
    pub fn new(
        id: U8Data,
        fractal: FractalDescriptor,
        max_iteration: u32,
        resolution: Resolution,
        range: Range,
    ) -> Self {
        Self {
            id,
            fractal,
            max_iteration,
            resolution,
            range,
        }
    }

    pub fn perform(
        &self,
    ) -> Result<(FragmentResult, FragmentResultData), Box<dyn std::error::Error>> {
        let (image_buffer, pixel_data) = self.initialize_buffers()?;
        let data = self.calculate_pixels(image_buffer)?;

        debug!("Calculated pixels for FragmentTask ID: {:?}", self.id);
        let fragment_result =
            FragmentResult::new(self.id.clone(), self.resolution, self.range, pixel_data);

        Ok((fragment_result, data))
    }

    fn initialize_buffers(
        &self,
    ) -> Result<(ImageBuffer<Rgb<u8>, Vec<u8>>, PixelData), Box<dyn std::error::Error>> {
        let width = self.resolution.nx as u32;
        let height = self.resolution.ny as u32;

        let image_buffer: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);
        let pixel_data = PixelData::new(self.id.count, width * height);

        Ok((image_buffer, pixel_data))
    }

    fn calculate_pixels(
        &self,
        image_buffer: ImageBuffer<Rgb<u8>, Vec<u8>>,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut data = Vec::new();

        for (x, y, _pixel) in image_buffer.enumerate_pixels() {
            let (mapped_x, mapped_y) = self.map_coordinates(x, y);
            let (zn, count) = self.calculate_fractal(mapped_x, mapped_y);

            let pixel_intensity =
                PixelIntensity::new(zn as f32, (count as f32) / self.max_iteration as f32);

            data.write_all(&pixel_intensity.zn.to_be_bytes())
                .map_err(|e| {
                    error!("Failed to write zn to pixel data: {}", e);
                    e
                })?;
            data.write_all(&pixel_intensity.count.to_be_bytes())
                .map_err(|e| {
                    error!("Failed to write count to pixel data: {}", e);
                    e
                })?;
        }

        Ok(data)
    }

    fn map_coordinates(&self, x: u32, y: u32) -> (f64, f64) {
        let Range { min, max } = &self.range;
        let mapped_x = min.x + (x as f64 / self.resolution.nx as f64) * (max.x - min.x);
        let mapped_y = min.y + (y as f64 / self.resolution.ny as f64) * (max.y - min.y);
        (mapped_x, mapped_y)
    }

    fn calculate_fractal(&self, x: f64, y: f64) -> (f64, f64) {
        match &self.fractal {
            FractalDescriptor::Julia(julia) => julia.generate(self.max_iteration, x, y),
            FractalDescriptor::Mandelbrot(mandelbrot) => {
                mandelbrot.generate(self.max_iteration, x, y)
            }
            FractalDescriptor::IteratedSinZ(iterated_sin_z) => {
                iterated_sin_z.generate(self.max_iteration, x, y)
            }
            FractalDescriptor::NewtonRaphsonZ3(newton_raphson_3) => {
                newton_raphson_3.generate(self.max_iteration, x, y)
            }
            FractalDescriptor::NewtonRaphsonZ4(newton_raphson_4) => {
                newton_raphson_4.generate(self.max_iteration, x, y)
            }
            FractalDescriptor::NovaNewtonRapshonZ3(nova_newton_raphson) => {
                nova_newton_raphson.generate(self.max_iteration, x, y)
            }
            FractalDescriptor::NovaNewtonRapshonZ4(nova_newton_raphson) => {
                nova_newton_raphson.generate(self.max_iteration, x, y)
            }
        }
    }
}

impl Fragment for FragmentTask {
    fn to_json(&self) -> Result<serde_json::Value, serde_json::Error> {
        let wrapped = serde_json::json!({ "FragmentTask": self });
        return serde_json::to_value(&wrapped);
    }

    fn from_json(fragment: &str) -> Result<Self, serde_json::Error> {
        let v: serde_json::Value = serde_json::from_str(fragment)?;
        serde_json::from_value(v["FragmentTask"].clone())
    }
}
