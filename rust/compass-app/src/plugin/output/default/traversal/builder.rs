use super::{geometry_output_format::TraversalOutputFormat, plugin::TraversalPlugin};
use crate::{
    app::compass::config::{
        builders::OutputPluginBuilder, compass_configuration_error::CompassConfigurationError,
        config_json_extension::ConfigJsonExtensions,
    },
    plugin::output::output_plugin::OutputPlugin,
};

pub struct TraversalPluginBuilder {}

impl OutputPluginBuilder for TraversalPluginBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Box<dyn OutputPlugin>, CompassConfigurationError> {
        let parent_key = String::from("traversal");
        let geometry_filename_key = String::from("geometry_file");
        let route_geometry_key = String::from("route");
        let tree_geometry_key = String::from("tree");

        let geometry_filename =
            parameters.get_config_string(geometry_filename_key, parent_key.clone())?;
        let route: Option<TraversalOutputFormat> =
            parameters.get_config_serde_optional(route_geometry_key, parent_key.clone())?;
        let tree: Option<TraversalOutputFormat> =
            parameters.get_config_serde_optional(tree_geometry_key, parent_key.clone())?;

        let geom_plugin = TraversalPlugin::from_file(&geometry_filename, route, tree)
            .map_err(CompassConfigurationError::PluginError)?;
        Ok(Box::new(geom_plugin))
    }
}
