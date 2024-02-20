use crate::models::fragments::{fragment_request::FragmentRequest, fragment_result::FragmentResult};

#[derive(Debug, Clone)]
pub struct RenderingData {
    pub result: FragmentResult,
    pub worker: String,
    pub iterations: Vec<f64>,
}
