use compass_core::model::{
    frontier::frontier_model::FrontierModel, traversal::traversal_model::TraversalModel,
};

use crate::plugin::{input::input_plugin::InputPlugin, output::output_plugin::OutputPlugin};

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

pub trait InputPluginBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Box<dyn InputPlugin>, CompassConfigurationError>;
}

pub trait OutputPluginBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Box<dyn OutputPlugin>, CompassConfigurationError>;
}
