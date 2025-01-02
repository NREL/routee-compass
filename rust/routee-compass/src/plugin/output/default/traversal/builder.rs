use super::{plugin::TraversalPlugin, traversal_output_format::TraversalOutputFormat};
use crate::{
    app::compass::{CompassConfigurationError, ConfigJsonExtensions},
    plugin::{
        output::{OutputPlugin, OutputPluginBuilder},
        PluginError,
    },
};
use std::sync::Arc;

/// Builds a plugin that can generate traversal outputs.
///
/// # Configuration
///
/// This plugin expects the following keys:
/// * `geometry_file` - the filename providing edge geometries
/// * `route` (optional) - traversal output format for the route result
/// * `tree` (optional) - traversal output format for the search tree result
///
/// See [TraversalOutputFormat] for information on the output formats supported.
///
/// [TraversalOutputFormat]: super::traversal_output_format::TraversalOutputFormat
///
/// # Example Configuration
///
/// ```toml
/// [[plugin.output_plugins]]
/// type = "traversal"
/// route = "geo_json"
/// tree = "geo_json"
/// geometry_input_file = "edges-geometries-enumerated.txt.gz"
/// ```
///
pub struct TraversalPluginBuilder {}

impl OutputPluginBuilder for TraversalPluginBuilder {
    /// builds the traversal output plugin, which allows users to configure how they want to
    /// output datasets related to the route plan and tree.
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn OutputPlugin>, CompassConfigurationError> {
        let parent_key = String::from("traversal");

        // let geometry_filename = parameters.get_config_path(&"geometry_input_file", &parent_key)?;
        let route: Option<TraversalOutputFormat> =
            parameters.get_config_serde_optional(&"route", &parent_key)?;
        let tree: Option<TraversalOutputFormat> =
            parameters.get_config_serde_optional(&"tree", &parent_key)?;

        let geom_plugin = TraversalPlugin::new(route, tree)
            .map_err(|e| PluginError::OutputPluginFailed { source: e })?;
        Ok(Arc::new(geom_plugin))
    }
}
