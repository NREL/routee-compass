//! Reads a table of grade values per edge id. These are recorded
//! directly to the traversal state vector as "leg_grade" values. If
//! no value was recorded, then a grade of zero is applied.

mod grade_configuration;
mod grade_traversal_builder;
mod grade_traversal_engine;
mod grade_traversal_model;
mod grade_traversal_service;

pub use grade_configuration::GradeConfiguration;
pub use grade_traversal_builder::GradeTraversalBuilder;
pub use grade_traversal_engine::GradeTraversalEngine;
pub use grade_traversal_model::GradeTraversalModel;
pub use grade_traversal_service::GradeTraversalService;
