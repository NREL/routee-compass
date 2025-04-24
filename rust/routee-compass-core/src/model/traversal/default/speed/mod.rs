mod speed_configuration;
mod speed_traversal_builder;
mod speed_traversal_engine;
mod speed_traversal_model;
mod speed_traversal_service;

pub use speed_configuration::SpeedConfiguration;
pub use speed_traversal_builder::SpeedTraversalBuilder;
pub use speed_traversal_engine::SpeedTraversalEngine;
pub use speed_traversal_model::SpeedTraversalModel;
pub use speed_traversal_service::SpeedLookupService;

/// input state value required to
// pub const EDGE_DISTANCE: &str = "edge_distance";

/// output state value for this edge's speed
pub const EDGE_SPEED: &str = "edge_speed";
