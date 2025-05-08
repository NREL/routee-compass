//! simple stateless model that calculates leg time from
//! leg speed and leg distance, also appending to the overall trip time.

mod time_configuration;
mod time_traversal_builder;
mod time_traversal_model;

pub use time_traversal_builder::TimeTraversalBuilder;
pub use time_traversal_model::TimeTraversalModel;
