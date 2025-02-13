use crate::plugin::input::InputPlugin;
use routee_compass_core::config::CompassConfigurationError;
use std::sync::Arc;

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
    ) -> Result<Arc<dyn InputPlugin>, CompassConfigurationError>;
}
