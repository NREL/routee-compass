use std::sync::Arc;

use super::{error::ConstraintModelError, ConstraintModelService};

/// A [`ConstraintModelBuilder`] takes a JSON object describing the configuration of a
/// constraint model and builds a [ConstraintModel].
///
/// A [`ConstraintModelBuilder`] instance should be an empty struct that implements
/// this trait.
///
/// [ConstraintModel]: routee_compass_core::model::constraint::ConstraintModel
pub trait ConstraintModelBuilder {
    /// Builds a [ConstraintModel] from JSON configuration.
    ///
    /// # Arguments
    ///
    /// * `parameters` - the contents of the "constraint" TOML config section
    ///
    /// # Returns
    ///
    /// A [ConstraintModel] designed to persist the duration of the CompassApp.
    ///
    /// [ConstraintModel]: routee_compass_core::model::constraint::ConstraintModel
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn ConstraintModelService>, ConstraintModelError>;
}
