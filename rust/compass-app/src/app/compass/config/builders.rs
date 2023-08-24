use compass_core::model::{
    frontier::frontier_model::FrontierModel, traversal::traversal_model::TraversalModel,
};

use super::compass_configuration_error::CompassConfigurationError;

pub trait TraversalModelBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Box<dyn TraversalModel>, CompassConfigurationError>;
}

pub trait FrontierModelBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Box<dyn FrontierModel>, CompassConfigurationError>;
}
