use super::compass_configuration_error::CompassConfigurationError;
use crate::plugin::{input::input_plugin::InputPlugin, output::output_plugin::OutputPlugin};
use compass_core::model::{
    frontier::frontier_model::FrontierModel, traversal::traversal_model::TraversalModel,
};
use std::sync::Arc;

pub trait TraversalModelBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModelService>, CompassConfigurationError>;
}

pub trait TraversalModelService: Send + Sync {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModel>, CompassConfigurationError>;
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
