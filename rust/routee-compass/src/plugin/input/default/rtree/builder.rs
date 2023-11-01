use crate::{
    app::compass::config::{
        builders::InputPluginBuilder, compass_configuration_error::CompassConfigurationError,
        config_json_extension::ConfigJsonExtensions,
    },
    plugin::input::input_plugin::InputPlugin,
};

use super::plugin::RTreePlugin;

pub struct VertexRTreeBuilder {}

impl InputPluginBuilder for VertexRTreeBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Box<dyn InputPlugin>, CompassConfigurationError> {
        let vertex_filename_key = String::from("vertices_file");
        let vertex_path = parameters.get_config_path(
            vertex_filename_key,
            String::from("Vertex RTree Input Plugin"),
        )?;
        let rtree =
            RTreePlugin::from_file(&vertex_path).map_err(CompassConfigurationError::PluginError)?;
        let m: Box<dyn InputPlugin> = Box::new(rtree);
        return Ok(m);
    }
}
