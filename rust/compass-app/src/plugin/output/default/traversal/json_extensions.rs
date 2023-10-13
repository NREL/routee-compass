use geo::{LineString, MultiLineString};
use wkt::ToWkt;

use crate::plugin::plugin_error::PluginError;

pub enum TraversalJsonField {
    RouteGeometry,
    TreeGeometry,
}

impl TraversalJsonField {
    pub fn as_str(self) -> &'static str {
        match self {
            TraversalJsonField::RouteGeometry => "geometry",
            TraversalJsonField::TreeGeometry => "tree_geometry",
        }
    }

    pub fn to_string(self) -> String {
        self.as_str().to_string()
    }
}

pub trait TraversalJsonExtensions {
    fn add_route_geometry(&mut self, geometry: LineString) -> Result<(), PluginError>;
    fn add_tree_geometry(&mut self, geometry: MultiLineString) -> Result<(), PluginError>;
    fn get_route_geometry_wkt(&self) -> Result<String, PluginError>;
}

impl TraversalJsonExtensions for serde_json::Value {
    fn add_route_geometry(&mut self, geometry: LineString) -> Result<(), PluginError> {
        let wkt = geometry.wkt_string();
        match self {
            serde_json::Value::Object(map) => {
                let json_string = serde_json::Value::String(wkt);
                map.insert(TraversalJsonField::RouteGeometry.to_string(), json_string);
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
                map.insert(TraversalJsonField::TreeGeometry.to_string(), json_string);
                Ok(())
            }
            _ => Err(PluginError::InputError(String::from(
                "OutputResult is not a JSON object",
            ))),
        }
    }

    fn get_route_geometry_wkt(&self) -> Result<String, PluginError> {
        let geometry = self
            .get(TraversalJsonField::RouteGeometry.as_str())
            .ok_or(PluginError::MissingField(
                TraversalJsonField::RouteGeometry.to_string(),
            ))?
            .as_str()
            .ok_or(PluginError::ParseError(
                TraversalJsonField::RouteGeometry.to_string(),
                String::from("string"),
            ))?
            .to_string();
        Ok(geometry)
    }
}
