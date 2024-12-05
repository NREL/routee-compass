use routee_compass_core::model::{map::map_error::MapError, network::EdgeId};

#[derive(thiserror::Error, Debug)]
pub enum MappingAppError {
    #[error(transparent)]
    MapError(#[from] MapError),
    #[error("expecting edge id {0} not found")]
    InvalidEdgeId(EdgeId),
}
