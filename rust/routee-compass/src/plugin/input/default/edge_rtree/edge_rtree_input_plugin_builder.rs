use super::edge_rtree_input_plugin::EdgeRtreeInputPlugin;
use crate::{
    app::compass::config::{
        builders::InputPluginBuilder, compass_configuration_error::CompassConfigurationError,
        config_json_extension::ConfigJsonExtensions,
    },
    plugin::input::input_plugin::InputPlugin,
};
use routee_compass_core::util::unit::{Distance, DistanceUnit};

pub struct EdgeRtreeInputPluginBuilder {}

impl InputPluginBuilder for EdgeRtreeInputPluginBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Box<dyn InputPlugin>, CompassConfigurationError> {
        let parent_key = String::from("edge_rtree");
        let linestring_file =
            parameters.get_config_path(String::from("geometry_input_file"), parent_key.clone())?;
        let road_class_file = parameters
            .get_config_path(String::from("road_class_input_file"), parent_key.clone())?;

        let distance_tolerance_option = parameters.get_config_serde_optional::<Distance>(
            String::from("distance_tolerance"),
            parent_key.clone(),
        )?;
        let distance_unit_option = parameters.get_config_serde_optional::<DistanceUnit>(
            String::from("distance_unit"),
            parent_key.clone(),
        )?;

        let plugin = EdgeRtreeInputPlugin::new(
            &road_class_file,
            &linestring_file,
            distance_tolerance_option,
            distance_unit_option,
        )?;
        Ok(Box::new(plugin))
    }
}
