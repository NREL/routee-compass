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
        let route_geometry_key = String::from("route_geometry");
        let tree_geometry_key = String::from("tree_geometry");
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
        let route_geometry = parameters
            .get(&route_geometry_key)
            .ok_or(CompassConfigurationError::ExpectedFieldForComponent(
                route_geometry_key.clone(),
                String::from("Geometry Output Plugin"),
            ))?
            .as_bool()
            .ok_or(CompassConfigurationError::ExpectedFieldWithType(
                route_geometry_key.clone(),
                String::from("String"),
            ))?;
        let tree_geometry = parameters
            .get(&tree_geometry_key)
            .ok_or(CompassConfigurationError::ExpectedFieldForComponent(
                tree_geometry_key.clone(),
                String::from("Geometry Output Plugin"),
            ))?
            .as_bool()
            .ok_or(CompassConfigurationError::ExpectedFieldWithType(
                tree_geometry_key.clone(),
                String::from("String"),
            ))?;
        let geom_plugin =
            GeometryPlugin::from_file(&geometry_filename, route_geometry, tree_geometry)
                .map_err(CompassConfigurationError::PluginError)?;
        Ok(Box::new(geom_plugin))
    }
}
