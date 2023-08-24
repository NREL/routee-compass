use std::path::PathBuf;

use crate::{
    app::compass::config::{
        builders::InputPluginBuilder, compass_configuration_error::CompassConfigurationError,
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
        let vertex_filename = parameters
            .get(&vertex_filename_key)
            .ok_or(CompassConfigurationError::ExpectedFieldForComponent(
                vertex_filename_key.clone(),
                String::from("Vertex RTree Input Plugin"),
            ))?
            .as_str()
            .map(String::from)
            .ok_or(CompassConfigurationError::ExpectedFieldWithType(
                vertex_filename_key.clone(),
                String::from("String"),
            ))?;
        let vertex_path = PathBuf::from(vertex_filename);
        let rtree =
            RTreePlugin::from_file(&vertex_path).map_err(CompassConfigurationError::PluginError)?;
        let m: Box<dyn InputPlugin> = Box::new(rtree);
        return Ok(m);
    }
}
