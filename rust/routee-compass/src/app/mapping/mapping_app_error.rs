use routee_compass_core::model::{map::map_error::MapError, road_network::edge_id::EdgeId};

#[derive(thiserror::Error, Debug)]
pub enum MappingAppError {
    #[error(transparent)]
    MapError(#[from] MapError),
    #[error("expecting edge id {0} not found")]
    InvalidEdgeId(EdgeId),
}
