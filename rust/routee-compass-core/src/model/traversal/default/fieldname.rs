//! defines common field names for the default traversal models
//! which are shared across models and used to declare the feature
//! dependency graph.
//!
//! ### naming convention
//!  - `edge_*` - state values for a single graph edge
//!  - `access_*` - state values for accessing a graph edge
//!  - `trip_*` - state values for a trip

/// state feature name for distance state values for a single graph edge
pub const EDGE_DISTANCE: &str = "edge_distance";
/// state feature name for accumulated trip distance to traverse this edge
pub const TRIP_DISTANCE: &str = "trip_distance";

/// state feature name for speed state values for a single graph edge
pub const EDGE_SPEED: &str = "edge_speed";

/// state feature name for time required to access this graph edge
pub const ACCESS_TIME: &str = "access_time";
/// state feature name for time required to traverse this graph edge
pub const EDGE_TIME: &str = "edge_time";
/// state feature name for accumulated trip time to traverse this edge
pub const TRIP_TIME: &str = "trip_time";

/// state feature name for grade state values for a single graph edge
pub const EDGE_GRADE: &str = "edge_grade";

/// state feature name for elevation gain accumulated  over a trip
pub const TRIP_ELEVATION_GAIN: &str = "trip_elevation_gain";
/// state feature name for elevation loss accumulated over a trip
pub const TRIP_ELEVATION_LOSS: &str = "trip_elevation_loss";
