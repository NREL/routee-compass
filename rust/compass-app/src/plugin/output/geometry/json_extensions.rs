use compass_core::model::graph::edge_id::EdgeId;
use geo::LineString;
use wkt::ToWkt;

use crate::plugin::plugin_error::PluginError;

pub enum GeometryJsonField {
    Path,
    EdgeId,
    EdgeCost,
    Geometry,
}

impl GeometryJsonField {
    pub fn as_str(self) -> &'static str {
        match self {
            GeometryJsonField::Path => "path",
            GeometryJsonField::EdgeId => "edge_id",
            GeometryJsonField::EdgeCost => "edge_cost",
            GeometryJsonField::Geometry => "geometry",
        }
    }

    pub fn as_string(self) -> String {
        self.as_str().to_string()
    }
}

pub trait GeometryJsonExtensions {
    // get the resulting path as a vector of edge ids
    fn get_edge_ids(&self) -> Result<Vec<EdgeId>, PluginError>;
    fn add_geometry(&mut self, geometry: LineString) -> Result<(), PluginError>;
    fn get_geometry_wkt(&self) -> Result<String, PluginError>;
}

impl GeometryJsonExtensions for serde_json::Value {
    fn get_edge_ids(&self) -> Result<Vec<EdgeId>, PluginError> {
        let path = self
            .get(GeometryJsonField::Path.as_str())
            .ok_or(PluginError::MissingField(GeometryJsonField::Path.as_str()))?;
        let edge_ids = path
            .as_array()
            .ok_or(PluginError::ParseError(GeometryJsonField::Path.as_str(), "array"))?
            .iter()
            .map(|edge| {
                edge.get(GeometryJsonField::EdgeId.as_str())
                    .ok_or(PluginError::MissingField(GeometryJsonField::EdgeId.as_str()))?
                    .as_u64()
                    .ok_or(PluginError::ParseError(GeometryJsonField::EdgeId.as_str(), "u64"))
                    .map(|id| EdgeId(id))
            })
            .collect::<Result<Vec<EdgeId>, PluginError>>()?;
        Ok(edge_ids)
    }

    fn add_geometry(&mut self, geometry: LineString) -> Result<(), PluginError> {
        let wkt = geometry.wkt_string();
        match self {
            serde_json::Value::Object(map) => {
                let json_string = serde_json::Value::String(wkt);
                map.insert(
                    GeometryJsonField::Geometry.as_string(),
                    json_string
                );
                Ok(())
            }
            _ => Err(PluginError::InputError("OutputResult is not a JSON object")),
        }
    }

    fn get_geometry_wkt(&self) -> Result<String, PluginError> {
        let geometry = self
            .get(GeometryJsonField::Geometry.as_str())
            .ok_or(PluginError::MissingField(GeometryJsonField::Geometry.as_str()))?
            .as_str()
            .ok_or(PluginError::ParseError(GeometryJsonField::Geometry.as_str(), "string"))?
            .to_string();
        Ok(geometry)
    }
}
