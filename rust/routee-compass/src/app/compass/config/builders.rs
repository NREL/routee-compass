use super::compass_configuration_error::CompassConfigurationError;
use crate::plugin::{input::input_plugin::InputPlugin, output::output_plugin::OutputPlugin};
use routee_compass_core::model::frontier::frontier_model::FrontierModel;

/// A [`FrontierModelBuilder`] takes a JSON object describing the configuration of a
/// frontier model and builds a [FrontierModel].
///
/// A [`FrontierModelBuilder`] instance should be an empty struct that implements
/// this trait.
///
/// [FrontierModel]: compass_core::model::frontier::frontier_model::FrontierModel
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
    /// [FrontierModel]: compass_core::model::frontier::frontier_model::FrontierModel
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Box<dyn FrontierModel>, CompassConfigurationError>;
}

/// A [`InputPluginBuilder`] takes a JSON object describing the configuration of an
/// input plugin and builds a [InputPlugin].
///
/// A [`InputPluginBuilder`] instance should be an empty struct that implements
/// this trait.
///
/// [InputPlugin]: compass_app::plugin::input::input_plugin::InputPlugin
pub trait InputPluginBuilder {
    /// Builds a [InputPlugin] from JSON configuration.
    ///
    /// # Arguments
    ///
    /// * `parameters` - the contents of an element in the "input_plugin" array TOML config section
    ///
    /// # Returns
    ///
    /// A [InputPlugin] designed to persist the duration of the CompassApp.
    ///
    /// [InputPlugin]: compass_app::plugin::input::input_plugin::InputPlugin
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Box<dyn InputPlugin>, CompassConfigurationError>;
}

/// A [`OutputPluginBuilder`] takes a JSON object describing the configuration of an
/// input plugin and builds a [OutputPlugin].
///
/// A [`OutputPluginBuilder`] instance should be an empty struct that implements
/// this trait.
///
/// [OutputPlugin]: compass_app::plugin::input::output_plugin::OutputPlugin
pub trait OutputPluginBuilder {
    /// Builds a [OutputPlugin] from JSON configuration.
    ///
    /// # Arguments
    ///
    /// * `parameters` - the contents of an element in the "output_plugin" array TOML config section
    ///
    /// # Returns
    ///
    /// A [OutputPlugin] designed to persist the duration of the CompassApp.
    ///
    /// [OutputPlugin]: compass_app::plugin::input::output_plugin::OutputPlugin
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Box<dyn OutputPlugin>, CompassConfigurationError>;
}
