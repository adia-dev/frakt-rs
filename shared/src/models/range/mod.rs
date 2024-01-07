use super::point::Point;

use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Range {
    pub min: Point,
    pub max: Point,
}
