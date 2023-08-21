use compass_core::model::traversal::traversal_model::TraversalModel;

use super::compass_configuration_error::CompassConfigurationError;

pub trait TraversalModelBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Box<dyn TraversalModel>, CompassConfigurationError>;
}
