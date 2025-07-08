use std::sync::Arc;

use crate::model::label::{
    label_model_error::LabelModelError, label_model_service::LabelModelService,
};

/// A [`LabelModelBuilder`] takes a JSON object describing the configuration of a
/// label model and builds a [LabelModelService].
///
/// A [`LabelModelBuilder`] instance should be an empty struct that implements
/// this trait. The builder is responsible for reading configuration parameters
/// and constructing the service that will be used to create label model instances
/// for individual queries.
///
/// The builder is called once during application startup to create the service,
/// which persists for the entire application lifetime. The service can then
/// create multiple model instances as needed for each query.
///
/// [LabelModelService]: super::label_model_service::LabelModelService
pub trait LabelModelBuilder {
    /// Builds a [LabelModelService] from JSON configuration.
    ///
    /// This method is called during application startup to create the service
    /// that will be used to build label model instances for individual queries.
    /// The configuration typically comes from the "label" section of the TOML
    /// configuration file.
    ///
    /// # Arguments
    ///
    /// * `parameters` - the contents of the "label" TOML config section
    ///
    /// # Returns
    ///
    /// A [LabelModelService] designed to persist the duration of the CompassApp,
    /// or an error if the configuration is invalid.
    ///
    /// # Examples
    ///
    /// A simple builder implementation might look like:
    /// ```ignore
    /// fn build(&self, parameters: &serde_json::Value) -> Result<Arc<dyn LabelModelService>, LabelModelError> {
    ///     // Parse any configuration parameters
    ///     let default_behavior = parameters.get("default_behavior")
    ///         .and_then(|v| v.as_str())
    ///         .unwrap_or("vertex_only");
    ///     
    ///     let service = MyLabelModelService {
    ///         default_behavior: default_behavior.to_string(),
    ///     };
    ///     Ok(Arc::new(service))
    /// }
    /// ```
    ///
    /// [LabelModelService]: super::label_model_service::LabelModelService
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn LabelModelService>, LabelModelError>;
}
