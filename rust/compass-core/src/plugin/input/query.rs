use geo;
use serde_json;

use crate::{plugin::plugin_error::PluginError, model::graph::vertex_id::VertexId};

pub trait InputQuery {
    fn get_origin_coordinate(&self) -> Result<geo::Coord<f64>, PluginError>;
    fn get_destination_coordinate(&self) -> Result<geo::Coord<f64>, PluginError>;
    fn add_origin_vertex(&mut self, vertex_id: VertexId) -> Result<(), PluginError>;
    fn add_destination_vertex(&mut self, vertex_id: VertexId) -> Result<(), PluginError>;
}

impl InputQuery for serde_json::Value {
    fn get_origin_coordinate(&self) -> Result<geo::Coord<f64>, PluginError> {
        let origin_latitude = self
            .get("origin_latitude")
            .ok_or(PluginError::MissingField("origin_latitude"))?
            .as_f64()
            .ok_or(PluginError::ParseError("origin_latitude", "f64"))?;
        let origin_longitude = self
            .get("origin_longitude")
            .ok_or(PluginError::MissingField("origin_longitude"))?
            .as_f64()
            .ok_or(PluginError::ParseError("origin_longitude", "f64"))?;
        Ok(geo::Coord::from((origin_latitude, origin_longitude)))
    }
    fn get_destination_coordinate(&self) -> Result<geo::Coord<f64>, PluginError> {
        let destination_latitude = self
            .get("destination_latitude")
            .ok_or(PluginError::MissingField("destination_latitude"))?
            .as_f64()
            .ok_or(PluginError::ParseError("destination_latitude", "f64"))?;
        let destination_longitude = self
            .get("destination_longitude")
            .ok_or(PluginError::MissingField("destination_longitude"))?
            .as_f64()
            .ok_or(PluginError::ParseError("destination_longitude", "f64"))?;
        Ok(geo::Coord::from((destination_latitude, destination_longitude)))
    }
    fn add_origin_vertex(&mut self, vertex_id: VertexId) -> Result<(), PluginError> {
        match self {
            serde_json::Value::Object(map) => {
                map.insert("origin_vertex".to_string(), serde_json::Value::from(vertex_id.0));
                Ok(())
            }
            _ => {
                Err(PluginError::InputError("InputQuery is not a JSON object"))
            }
        }
    }
    fn add_destination_vertex(&mut self, vertex_id: VertexId) -> Result<(), PluginError> {
        match self {
            serde_json::Value::Object(map) => {
                map.insert("destination_vertex".to_string(), serde_json::Value::from(vertex_id.0));
                Ok(())
            }
            _ => {
                Err(PluginError::InputError("InputQuery is not a JSON object"))
            }
        }
    }

}
