use crate::{
    app::compass::config::{
        builders::OutputPluginBuilder, compass_configuration_error::CompassConfigurationError,
    },
    plugin::output::output_plugin::OutputPlugin,
};

use super::plugin::GeometryPlugin;

pub struct GeometryPluginBuilder {}

impl OutputPluginBuilder for GeometryPluginBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Box<dyn OutputPlugin>, CompassConfigurationError> {
        let geometry_filename_key = String::from("edge_file");
        let geometry_filename = parameters
            .get(&geometry_filename_key)
            .ok_or(CompassConfigurationError::ExpectedFieldForComponent(
                geometry_filename_key.clone(),
                String::from("Geometry Output Plugin"),
            ))?
            .as_str()
            .map(String::from)
            .ok_or(CompassConfigurationError::ExpectedFieldWithType(
                geometry_filename_key.clone(),
                String::from("String"),
            ))?;
        let geom_plugin = GeometryPlugin::from_file(&geometry_filename)
            .map_err(CompassConfigurationError::PluginError)?;
        Ok(Box::new(geom_plugin))
    }
}
