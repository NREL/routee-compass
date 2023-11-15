use routee_compass_core::util::unit::{Distance, DistanceUnit};

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
        let parent_key = String::from("Vertex RTree Input Plugin");
        let vertex_filename_key = String::from("vertices_input_file");
        let vertex_path = parameters.get_config_path(vertex_filename_key, parent_key.clone())?;
        let tolerance_distance = parameters.get_config_serde_optional::<Distance>(
            String::from("distance_tolerance"),
            parent_key.clone(),
        )?;
        let distance_unit = parameters.get_config_serde_optional::<DistanceUnit>(
            String::from("distance_unit"),
            parent_key.clone(),
        )?;
        let rtree = RTreePlugin::new(&vertex_path, tolerance_distance, distance_unit)
            .map_err(CompassConfigurationError::PluginError)?;
        let m: Box<dyn InputPlugin> = Box::new(rtree);
        Ok(m)
    }
}
