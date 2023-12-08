use std::sync::Arc;

use super::{
    frontier_model_error::FrontierModelError, frontier_model_service::FrontierModelService,
};

/// A [`FrontierModelBuilder`] takes a JSON object describing the configuration of a
/// frontier model and builds a [FrontierModel].
///
/// A [`FrontierModelBuilder`] instance should be an empty struct that implements
/// this trait.
///
/// [FrontierModel]: routee_compass_core::model::frontier::frontier_model::FrontierModel
pub trait FrontierModelBuilder {
    /// Builds a [FrontierModel] from JSON configuration.
    ///
    /// # Arguments
    ///
    /// * `parameters` - the contents of the "frontier" TOML config section
    ///
    /// # Returns
    ///
    /// A [FrontierModel] designed to persist the duration of the CompassApp.
    ///
    /// [FrontierModel]: routee_compass_core::model::frontier::frontier_model::FrontierModel
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn FrontierModelService>, FrontierModelError>;
}
