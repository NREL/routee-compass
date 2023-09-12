use geo::{LineString, MultiLineString};
use wkt::ToWkt;

use crate::plugin::plugin_error::PluginError;

pub enum GeometryJsonField {
    RouteGeometry,
    TreeGeometry,
}

impl GeometryJsonField {
    pub fn as_str(self) -> &'static str {
        match self {
            GeometryJsonField::RouteGeometry => "geometry",
            GeometryJsonField::TreeGeometry => "tree_geometry",
        }
    }

    pub fn to_string(self) -> String {
        self.as_str().to_string()
    }
}

pub trait GeometryJsonExtensions {
    fn add_route_geometry(&mut self, geometry: LineString) -> Result<(), PluginError>;
    fn add_tree_geometry(&mut self, geometry: MultiLineString) -> Result<(), PluginError>;
    fn get_route_geometry_wkt(&self) -> Result<String, PluginError>;
}

impl GeometryJsonExtensions for serde_json::Value {
    fn add_route_geometry(&mut self, geometry: LineString) -> Result<(), PluginError> {
        let wkt = geometry.wkt_string();
        match self {
            serde_json::Value::Object(map) => {
                let json_string = serde_json::Value::String(wkt);
                map.insert(GeometryJsonField::RouteGeometry.to_string(), json_string);
                Ok(())
            }
            _ => Err(PluginError::InputError(String::from(
                "OutputResult is not a JSON object",
            ))),
        }
    }

    fn add_tree_geometry(&mut self, geometry: MultiLineString) -> Result<(), PluginError> {
        let wkt = geometry.wkt_string();
        match self {
            serde_json::Value::Object(map) => {
                let json_string = serde_json::Value::String(wkt);
                map.insert(GeometryJsonField::TreeGeometry.to_string(), json_string);
                Ok(())
            }
            _ => Err(PluginError::InputError(String::from(
                "OutputResult is not a JSON object",
            ))),
        }
    }

    fn get_route_geometry_wkt(&self) -> Result<String, PluginError> {
        let geometry = self
            .get(GeometryJsonField::RouteGeometry.as_str())
            .ok_or(PluginError::MissingField(
                GeometryJsonField::RouteGeometry.to_string(),
            ))?
            .as_str()
            .ok_or(PluginError::ParseError(
                GeometryJsonField::RouteGeometry.to_string(),
                String::from("string"),
            ))?
            .to_string();
        Ok(geometry)
    }

}
