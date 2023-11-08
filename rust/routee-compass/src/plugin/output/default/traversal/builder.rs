use super::{plugin::TraversalPlugin, traversal_output_format::TraversalOutputFormat};
use crate::{
    app::compass::config::{
        builders::OutputPluginBuilder, compass_configuration_error::CompassConfigurationError,
        config_json_extension::ConfigJsonExtensions,
    },
    plugin::output::output_plugin::OutputPlugin,
};

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
    ) -> Result<Box<dyn OutputPlugin>, CompassConfigurationError> {
        let parent_key = String::from("traversal");
        let geometry_filename_key = String::from("geometry_input_file");
        let route_geometry_key = String::from("route");
        let tree_geometry_key = String::from("tree");

        let geometry_filename =
            parameters.get_config_path(geometry_filename_key, parent_key.clone())?;
        let route: Option<TraversalOutputFormat> =
            parameters.get_config_serde_optional(route_geometry_key, parent_key.clone())?;
        let tree: Option<TraversalOutputFormat> =
            parameters.get_config_serde_optional(tree_geometry_key, parent_key.clone())?;

        let geom_plugin = TraversalPlugin::from_file(&geometry_filename, route, tree)?;
        Ok(Box::new(geom_plugin))
    }
}
