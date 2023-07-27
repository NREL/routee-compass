use geo;
use serde_json;

use compass_core::model::graph::vertex_id::VertexId;

use crate::plugin::plugin_error::PluginError;

pub enum InputField {
    OriginX,
    OriginY,
    DestinationX,
    DestinationY,
    OriginVertex,
    DestinationVertex,
}

impl InputField {
    pub fn to_str(&self) -> &'static str {
        match self {
            InputField::OriginX => "origin_x",
            InputField::OriginY => "origin_y",
            InputField::DestinationX => "destination_x",
            InputField::DestinationY => "destination_y",
            InputField::OriginVertex => "origin_vertex",
            InputField::DestinationVertex => "destination_vertex",
        }
    }
    pub fn to_string(&self) -> String {
        self.to_str().to_string()
    }
}

pub trait InputQuery {
    fn get_origin_coordinate(&self) -> Result<geo::Coord<f64>, PluginError>;
    fn get_destination_coordinate(&self) -> Result<geo::Coord<f64>, PluginError>;
    fn add_origin_vertex(&mut self, vertex_id: VertexId) -> Result<(), PluginError>;
    fn add_destination_vertex(&mut self, vertex_id: VertexId) -> Result<(), PluginError>;
}

impl InputQuery for serde_json::Value {
    fn get_origin_coordinate(&self) -> Result<geo::Coord<f64>, PluginError> {
        let origin_x = self
            .get(InputField::OriginX.to_str())
            .ok_or(PluginError::MissingField(InputField::OriginX.to_str()))?
            .as_f64()
            .ok_or(PluginError::ParseError(InputField::OriginX.to_str(), "f64"))?;
        let origin_y = self
            .get(InputField::OriginY.to_str())
            .ok_or(PluginError::MissingField(InputField::OriginY.to_str()))?
            .as_f64()
            .ok_or(PluginError::ParseError(InputField::OriginY.to_str(), "f64"))?;
        Ok(geo::Coord::from((origin_x, origin_y)))
    }
    fn get_destination_coordinate(&self) -> Result<geo::Coord<f64>, PluginError> {
        let destination_x = self
            .get(InputField::DestinationX.to_str())
            .ok_or(PluginError::MissingField(InputField::DestinationX.to_str()))?
            .as_f64()
            .ok_or(PluginError::ParseError(
                InputField::DestinationX.to_str(),
                "f64",
            ))?;
        let destination_y = self
            .get(InputField::DestinationY.to_str())
            .ok_or(PluginError::MissingField(InputField::DestinationY.to_str()))?
            .as_f64()
            .ok_or(PluginError::ParseError(
                InputField::DestinationY.to_str(),
                "f64",
            ))?;
        Ok(geo::Coord::from((destination_x, destination_y)))
    }
    fn add_origin_vertex(&mut self, vertex_id: VertexId) -> Result<(), PluginError> {
        match self {
            serde_json::Value::Object(map) => {
                map.insert(
                    InputField::OriginVertex.to_string(),
                    serde_json::Value::from(vertex_id.0),
                );
                Ok(())
            }
            _ => Err(PluginError::InputError("InputQuery is not a JSON object")),
        }
    }
    fn add_destination_vertex(&mut self, vertex_id: VertexId) -> Result<(), PluginError> {
        match self {
            serde_json::Value::Object(map) => {
                map.insert(
                    InputField::DestinationVertex.to_string(),
                    serde_json::Value::from(vertex_id.0),
                );
                Ok(())
            }
            _ => Err(PluginError::InputError("InputQuery is not a JSON object")),
        }
    }
}
