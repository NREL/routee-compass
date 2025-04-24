//! simple stateless model that calculates leg time from
//! leg speed and leg distance, also appending to the overall trip time.

mod time_configuration;
mod time_traversal_builder;
mod time_traversal_model;

pub use time_traversal_builder::TimeTraversalBuilder;
pub use time_traversal_model::TimeTraversalModel;

/// input state feature name for distance state values for a single graph edge.
/// used to compute elevation gain/loss.
pub const EDGE_DISTANCE: &str = "edge_distance";
/// input state feature name for distance state values for a single graph edge
pub const EDGE_SPEED: &str = "edge_speed";
/// output state feature name for time required to traverse this graph edge
pub const EDGE_TIME: &str = "edge_time";
/// output state feature name for accumulated trip time to traverse this edge
pub const TRIP_TIME: &str = "trip_time";
