use crate::{app::compass::CompassAppError, plugin::PluginError};

use super::{InputField, InputPluginError};
use geo;
use routee_compass_core::model::network::{EdgeId, VertexId};
use serde_json::{self, json};

pub trait InputJsonExtensions {
    fn get_origin_coordinate(&self) -> Result<geo::Coord<f32>, InputPluginError>;
    fn get_destination_coordinate(&self) -> Result<Option<geo::Coord<f32>>, InputPluginError>;
    fn add_origin_vertex(&mut self, vertex_id: VertexId) -> Result<(), InputPluginError>;
    fn add_destination_vertex(&mut self, vertex_id: VertexId) -> Result<(), InputPluginError>;
    fn add_origin_edge(&mut self, edge_id: EdgeId) -> Result<(), InputPluginError>;
    fn add_destination_edge(&mut self, edge_id: EdgeId) -> Result<(), InputPluginError>;
    fn get_origin_vertex(&self) -> Result<VertexId, InputPluginError>;
    fn get_destination_vertex(&self) -> Result<Option<VertexId>, InputPluginError>;
    fn get_origin_edge(&self) -> Result<EdgeId, InputPluginError>;
    fn get_destination_edge(&self) -> Result<Option<EdgeId>, InputPluginError>;
    fn get_grid_search(&self) -> Option<&serde_json::Value>;
    fn add_query_weight_estimate(&mut self, weight: f64) -> Result<(), InputPluginError>;
    fn get_query_weight_estimate(&self) -> Result<Option<f64>, CompassAppError>;
}

impl InputJsonExtensions for serde_json::Value {
    fn get_origin_coordinate(&self) -> Result<geo::Coord<f32>, InputPluginError> {
        let origin_x = self
            .get(InputField::OriginX.to_str())
            .ok_or(InputPluginError::MissingExpectedQueryField(
                InputField::OriginX,
            ))?
            .as_f64()
            .ok_or_else(|| {
                InputPluginError::QueryFieldHasInvalidType(InputField::OriginX, String::from("f64"))
            })?;
        let origin_y = self
            .get(InputField::OriginY.to_str())
            .ok_or(InputPluginError::MissingExpectedQueryField(
                InputField::OriginY,
            ))?
            .as_f64()
            .ok_or_else(|| {
                InputPluginError::QueryFieldHasInvalidType(InputField::OriginY, String::from("f64"))
            })?;
        Ok(geo::Coord::from((origin_x as f32, origin_y as f32)))
    }
    fn get_destination_coordinate(&self) -> Result<Option<geo::Coord<f32>>, InputPluginError> {
        let x_field = InputField::DestinationX;
        let y_field = InputField::DestinationY;
        let x_opt = self.get(x_field.to_str());
        let y_opt = self.get(y_field.to_str());
        match (x_opt, y_opt) {
            (None, None) => Ok(None),
            (None, Some(_)) => Err(InputPluginError::MissingQueryFieldPair(y_field, x_field)),
            (Some(_), None) => Err(InputPluginError::MissingQueryFieldPair(x_field, y_field)),
            (Some(x_json), Some(y_json)) => {
                let x = x_json.as_f64().ok_or_else(|| {
                    InputPluginError::QueryFieldHasInvalidType(x_field, String::from("f64"))
                })?;
                let y = y_json.as_f64().ok_or_else(|| {
                    InputPluginError::QueryFieldHasInvalidType(y_field, String::from("f64"))
                })?;
                Ok(Some(geo::Coord::from((x as f32, y as f32))))
            }
        }
    }
    fn add_origin_vertex(&mut self, vertex_id: VertexId) -> Result<(), InputPluginError> {
        match self {
            serde_json::Value::Object(map) => {
                map.insert(
                    InputField::OriginVertex.to_string(),
                    serde_json::Value::from(vertex_id.0),
                );
                Ok(())
            }
            _ => Err(InputPluginError::UnexpectedQueryStructure(String::from(
                "InputQuery is not a JSON object",
            ))),
        }
    }
    fn add_destination_vertex(&mut self, vertex_id: VertexId) -> Result<(), InputPluginError> {
        match self {
            serde_json::Value::Object(map) => {
                map.insert(
                    InputField::DestinationVertex.to_string(),
                    serde_json::Value::from(vertex_id.0),
                );
                Ok(())
            }
            _ => Err(InputPluginError::UnexpectedQueryStructure(String::from(
                "InputQuery is not a JSON object",
            ))),
        }
    }

    fn get_origin_vertex(&self) -> Result<VertexId, InputPluginError> {
        self.get(InputField::OriginVertex.to_str())
            .ok_or(InputPluginError::MissingExpectedQueryField(
                InputField::OriginVertex,
            ))?
            .as_u64()
            .map(|v| VertexId(v as usize))
            .ok_or_else(|| {
                InputPluginError::QueryFieldHasInvalidType(
                    InputField::OriginVertex,
                    String::from("u64"),
                )
            })
    }

    fn get_destination_vertex(&self) -> Result<Option<VertexId>, InputPluginError> {
        match self.get(InputField::DestinationVertex.to_str()) {
            None => Ok(None),
            Some(v) => v
                .as_u64()
                .map(|v| Some(VertexId(v as usize)))
                .ok_or_else(|| {
                    InputPluginError::QueryFieldHasInvalidType(
                        InputField::DestinationVertex,
                        String::from("u64"),
                    )
                }),
        }
    }

    fn get_origin_edge(&self) -> Result<EdgeId, InputPluginError> {
        self.get(InputField::OriginEdge.to_str())
            .ok_or(InputPluginError::MissingExpectedQueryField(
                InputField::OriginEdge,
            ))?
            .as_u64()
            .map(|v| EdgeId(v as usize))
            .ok_or_else(|| {
                InputPluginError::QueryFieldHasInvalidType(
                    InputField::OriginEdge,
                    String::from("u64"),
                )
            })
    }

    fn get_destination_edge(&self) -> Result<Option<EdgeId>, InputPluginError> {
        match self.get(InputField::DestinationEdge.to_str()) {
            None => Ok(None),
            Some(v) => v.as_u64().map(|v| Some(EdgeId(v as usize))).ok_or_else(|| {
                InputPluginError::QueryFieldHasInvalidType(
                    InputField::OriginEdge,
                    String::from("u64"),
                )
            }),
        }
    }
    fn get_grid_search(&self) -> Option<&serde_json::Value> {
        self.get(InputField::GridSearch.to_str())
    }

    fn add_origin_edge(&mut self, edge_id: EdgeId) -> Result<(), InputPluginError> {
        match self {
            serde_json::Value::Object(map) => {
                map.insert(
                    InputField::OriginEdge.to_string(),
                    serde_json::Value::from(edge_id.0),
                );
                Ok(())
            }
            _ => Err(InputPluginError::UnexpectedQueryStructure(String::from(
                "InputQuery is not a JSON object",
            ))),
        }
    }

    fn add_destination_edge(&mut self, edge_id: EdgeId) -> Result<(), InputPluginError> {
        match self {
            serde_json::Value::Object(map) => {
                map.insert(
                    InputField::DestinationEdge.to_string(),
                    serde_json::Value::from(edge_id.0),
                );
                Ok(())
            }
            _ => Err(InputPluginError::UnexpectedQueryStructure(String::from(
                "InputQuery is not a JSON object",
            ))),
        }
    }

    fn add_query_weight_estimate(&mut self, weight: f64) -> Result<(), InputPluginError> {
        match self {
            serde_json::Value::Object(map) => {
                map.insert(InputField::QueryWeightEstimate.to_string(), json!(weight));
                Ok(())
            }
            _ => Err(InputPluginError::UnexpectedQueryStructure(String::from(
                "InputQuery is not a JSON object",
            ))),
        }
    }

    fn get_query_weight_estimate(&self) -> Result<Option<f64>, CompassAppError> {
        match self.get(InputField::QueryWeightEstimate.to_str()) {
            None => Ok(None),
            Some(v) => v.as_f64().map(Some).ok_or_else(|| {
                let ipe = InputPluginError::QueryFieldHasInvalidType(
                    InputField::QueryWeightEstimate,
                    String::from("f64"),
                );
                let pe = PluginError::InputPluginFailed { source: ipe };
                CompassAppError::PluginError(pe)
            }),
        }
    }
}

// pub type DecodeOp<T> = Box<dyn Fn(&serde_json::Value) -> Option<T>>;

// fn get_from_json<T>(
//     value: &serde_json::Value,
//     field: InputField,
//     op: DecodeOp<T>,
// ) -> Result<T, InputPluginError> {
//     let at_field = value.get(field.to_string());
//     match at_field {
//         None => Err(InputPluginError::MissingField(field.to_string())),
//         Some(v) => op(v).ok_or_else( ||InputPluginError::QueryFieldHasInvalidType(field.to_string(), ())),
//     };
// }

// fn get_f64(v: &serde_json::Value) -> Result<f64, InputPluginError> {

//     get_from_json(v, field, |v| v.as_f64())
//     v.as_f64().ok_or_else( ||InputPluginError::QueryFieldHasInvalidType((), ())
// }
