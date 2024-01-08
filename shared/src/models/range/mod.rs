use super::point::Point;

use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Range {
    pub min: Point,
    pub max: Point,
}

impl Range {
    pub fn new(min: Point, max: Point) -> Self {
        Self { min, max }
    }
}
