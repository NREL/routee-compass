//! simple stateless model that calculates leg time from
//! leg speed and leg distance, also appending to the overall trip time.

mod time_traversal_builder;
mod time_traversal_model;
mod time_traversal_config;

pub use time_traversal_builder::TimeTraversalBuilder;
pub use time_traversal_model::TimeTraversalModel;
pub use time_traversal_config::TimeTraversalConfig;