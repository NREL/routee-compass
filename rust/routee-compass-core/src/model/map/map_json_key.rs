use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "snake_case")]
pub enum MapJsonKey {
    OriginX,
    OriginY,
    DestinationX,
    DestinationY,
    OriginVertex,
    DestinationVertex,
    OriginEdge,
    DestinationEdge,
}

impl Display for MapJsonKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use MapJsonKey as I;
        let s = match self {
            I::OriginX => "origin_x",
            I::OriginY => "origin_y",
            I::DestinationX => "destination_x",
            I::DestinationY => "destination_y",
            I::OriginVertex => "origin_vertex",
            I::DestinationVertex => "destination_vertex",
            I::OriginEdge => "origin_edge",
            I::DestinationEdge => "destination_edge",
        };
        write!(f, "{}", s)
    }
}
