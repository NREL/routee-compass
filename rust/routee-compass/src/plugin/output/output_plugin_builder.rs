use super::OutputPlugin;
use crate::app::compass::CompassComponentError;
use std::sync::Arc;

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
    ) -> Result<Arc<dyn OutputPlugin>, CompassComponentError>;
}
