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
    OriginEdgeList,
    OriginEdge,
    DestinationEdgeList,
    DestinationEdge,
}

impl MapJsonKey {
    pub const fn as_str(&self) -> &'static str {
        match self {
            MapJsonKey::OriginX => "origin_x",
            MapJsonKey::OriginY => "origin_y",
            MapJsonKey::DestinationX => "destination_x",
            MapJsonKey::DestinationY => "destination_y",
            MapJsonKey::OriginVertex => "origin_vertex",
            MapJsonKey::DestinationVertex => "destination_vertex",
            MapJsonKey::OriginEdgeList => "origin_edge_list",
            MapJsonKey::OriginEdge => "origin_edge",
            MapJsonKey::DestinationEdgeList => "destination_edge_list",
            MapJsonKey::DestinationEdge => "destination_edge",
        }
    }
}

impl Display for MapJsonKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
