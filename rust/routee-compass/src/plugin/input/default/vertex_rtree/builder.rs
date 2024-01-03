use std::sync::Arc;

use routee_compass_core::model::unit::{Distance, DistanceUnit};

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
    ) -> Result<Arc<dyn InputPlugin>, CompassConfigurationError> {
        let parent_key = String::from("Vertex RTree Input Plugin");
        let vertex_path = parameters.get_config_path(&"vertices_input_file", &parent_key)?;
        let tolerance_distance =
            parameters.get_config_serde_optional::<Distance>(&"distance_tolerance", &parent_key)?;
        let distance_unit =
            parameters.get_config_serde_optional::<DistanceUnit>(&"distance_unit", &parent_key)?;
        let rtree = RTreePlugin::new(&vertex_path, tolerance_distance, distance_unit)
            .map_err(CompassConfigurationError::PluginError)?;
        let m: Arc<dyn InputPlugin> = Arc::new(rtree);
        Ok(m)
    }
}
