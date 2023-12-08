use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize)]
#[serde(rename = "snake_case")]
pub enum Direction {
    Forward,
    Reverse,
}
