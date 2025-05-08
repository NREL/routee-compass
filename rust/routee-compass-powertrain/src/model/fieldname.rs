//! defines common field names for the powertrain traversal models
//! which are shared across models and used to declare the feature
//! dependency graph.
//!
//! ### naming convention
//!  - `edge_*` - state values for a single graph edge
//!  - `access_*` - state values for accessing a graph edge
//!  - `trip_*` - state values for a trip

pub const EDGE_ENERGY_LIQUID: &'static str = "edge_energy_liquid";
pub const TRIP_ENERGY_LIQUID: &'static str = "trip_energy_liquid";
pub const EDGE_ENERGY_ELECTRIC: &'static str = "edge_energy_electric";
pub const TRIP_ENERGY_ELECTRIC: &'static str = "trip_energy_electric";
pub const TRIP_SOC: &'static str = "trip_soc";
pub use routee_compass_core::model::traversal::default::fieldname::*;
