//! Reads a table of temperature values per edge id. These are recorded
//! directly to the traversal state vector as "leg_temperature" values. If
//! no value was recorded, then a temperature of zero is applied.

mod temperature_traversal_builder;
mod temperature_traversal_model;
mod temperature_traversal_service;

pub use temperature_traversal_builder::TemperatureTraversalBuilder;
pub use temperature_traversal_model::TemperatureTraversalModel;
pub use temperature_traversal_service::TemperatureTraversalService;
