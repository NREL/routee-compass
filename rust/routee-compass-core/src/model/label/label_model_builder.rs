use std::sync::Arc;

use crate::model::label::{
    label_model_error::LabelModelError, label_model_service::LabelModelService,
};

/// A [`LabelModelBuilder`] takes a JSON object describing the configuration of a
/// label model and builds a [LabelModel].
///
/// A [`LabelModelBuilder`] instance should be an empty struct that implements
/// this trait.
///
/// [LabelModel]: routee_compass_core::model::label::LabelModel
pub trait LabelModelBuilder {
    /// Builds a [LabelModel] from JSON configuration.
    ///
    /// # Arguments
    ///
    /// * `parameters` - the contents of the "label" TOML config section
    ///
    /// # Returns
    ///
    /// A [LabelModel] designed to persist the duration of the CompassApp.
    ///
    /// [LabelModel]: routee_compass_core::model::label::LabelModel
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn LabelModelService>, LabelModelError>;
}
