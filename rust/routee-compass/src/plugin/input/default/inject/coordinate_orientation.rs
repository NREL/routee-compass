use clap::ValueEnum;
use serde::{Deserialize, Serialize};

/// when performing a spatial inject, the orientation determines which coordinate
/// of a query is used for the spatial intersection query.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum CoordinateOrientation {
    Origin,
    Destination,
}
