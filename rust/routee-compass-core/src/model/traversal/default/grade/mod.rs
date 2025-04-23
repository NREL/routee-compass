//! Reads a table of grade values per edge id. These are recorded
//! directly to the traversal state vector as "leg_grade" values. If
//! no value was recorded, then a grade of zero is applied.
//!
//! Additionally stores overall elevation metrics as "trip_elevation_gain" and
//! "trip_elevation_loss".

mod elevation_change;
mod grade_configuration;
mod grade_traversal_builder;
mod grade_traversal_engine;
mod grade_traversal_model;
mod grade_traversal_service;

pub use elevation_change::ElevationChange;
pub use grade_configuration::GradeConfiguration;
pub use grade_traversal_builder::GradeTraversalBuilder;
pub use grade_traversal_engine::GradeTraversalEngine;
pub use grade_traversal_model::GradeTraversalModel;
pub use grade_traversal_service::GradeTraversalService;

/// input state feature name for distance state values for a single graph edge.
/// used to compute elevation gain/loss.
pub const LEG_DISTANCE: &str = "leg_distance";
/// output state feature name for grade state values for a single graph edge
pub const LEG_GRADE: &str = "leg_grade";
/// output state feature name for elevation gain accumulated  over a trip
pub const TRIP_ELEVATION_GAIN: &str = "trip_elevation_gain";
/// output state feature name for elevation loss accumulated over a trip
pub const TRIP_ELEVATION_LOSS: &str = "trip_elevation_loss";
