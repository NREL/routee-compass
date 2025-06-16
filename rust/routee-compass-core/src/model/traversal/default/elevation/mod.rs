//! simple stateless model that calculates leg time from
//! leg speed and leg distance, also appending to the overall trip time.

mod elevation_change;
mod elevation_traversal_builder;
mod elevation_traversal_model;

pub use elevation_change::ElevationChange;
pub use elevation_traversal_builder::ElevationTraversalBuilder;
pub use elevation_traversal_model::ElevationTraversalModel;
