//! defines common field names for the powertrain traversal models
//! which are shared across models and used to declare the feature
//! dependency graph.
//!
//! also exports the default field names from the core library.
//!
//! ### naming convention
//!  - `edge_*` - state values for a single graph edge
//!  - `access_*` - state values for accessing a graph edge
//!  - `trip_*` - state values for a trip
//!
//! ### model types
//!
//! - ICE
//!   - uses "trip_energy_liquid" and "edge_energy_liquid" for energy consumption
//! - BEV
//!   - uses "trip_energy_electric" and "edge_energy_electric" for energy consumption
//!   - adds "trip_soc" for state of charge percentage
//! - PHEV
//!   - uses all of the above

/// state feature name for liquid fuel state values for a single graph edge
pub const EDGE_ENERGY_LIQUID: &str = "edge_energy_liquid";
/// state feature name for accumulated liquid fuel state values at some graph edge
pub const TRIP_ENERGY_LIQUID: &str = "trip_energy_liquid";
/// state feature name for battery fuel state values for a single graph edge
pub const EDGE_ENERGY_ELECTRIC: &str = "edge_energy_electric";
/// state feature name for accumulated battery fuel state values at some graph edge
pub const TRIP_ENERGY_ELECTRIC: &str = "trip_energy_electric";
/// overall trip state of charge percentage value
pub const TRIP_SOC: &str = "trip_soc";

pub const BATTERY_CAPACITY: &str = "battery_capacity";
pub use routee_compass_core::model::traversal::default::fieldname::*;
