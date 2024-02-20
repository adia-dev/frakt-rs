use crate::models::fragments::{fragment_request::FragmentRequest, fragment_result::FragmentResult};

#[derive(Debug, Clone)]
pub struct RenderingData {
    pub result: FragmentResult,
    pub worker: String,
    pub pixels: Vec<(u8, u8, u8)>,
    pub counts: Vec<f64>,
}
