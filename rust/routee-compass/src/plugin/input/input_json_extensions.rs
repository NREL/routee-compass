use super::input_field::InputField;
use crate::plugin::plugin_error::PluginError;
use geo;
use routee_compass_core::model::road_network::{edge_id::EdgeId, vertex_id::VertexId};
use serde_json;

pub trait InputJsonExtensions {
    fn get_origin_coordinate(&self) -> Result<geo::Coord<f64>, PluginError>;
    fn get_destination_coordinate(&self) -> Result<Option<geo::Coord<f64>>, PluginError>;
    fn add_origin_vertex(&mut self, vertex_id: VertexId) -> Result<(), PluginError>;
    fn add_destination_vertex(&mut self, vertex_id: VertexId) -> Result<(), PluginError>;
    fn add_origin_edge(&mut self, edge_id: EdgeId) -> Result<(), PluginError>;
    fn add_destination_edge(&mut self, edge_id: EdgeId) -> Result<(), PluginError>;
    fn get_origin_vertex(&self) -> Result<VertexId, PluginError>;
    fn get_destination_vertex(&self) -> Result<Option<VertexId>, PluginError>;
    fn get_origin_edge(&self) -> Result<EdgeId, PluginError>;
    fn get_destination_edge(&self) -> Result<Option<EdgeId>, PluginError>;
    fn get_grid_search(&self) -> Option<&serde_json::Value>;
}

impl InputJsonExtensions for serde_json::Value {
    fn get_origin_coordinate(&self) -> Result<geo::Coord<f64>, PluginError> {
        let origin_x = self
            .get(InputField::OriginX.to_string())
            .ok_or(PluginError::MissingField(InputField::OriginX.to_string()))?
            .as_f64()
            .ok_or(PluginError::ParseError(
                InputField::OriginX.to_string(),
                String::from("f64"),
            ))?;
        let origin_y = self
            .get(InputField::OriginY.to_string())
            .ok_or(PluginError::MissingField(InputField::OriginY.to_string()))?
            .as_f64()
            .ok_or(PluginError::ParseError(
                InputField::OriginY.to_string(),
                String::from("f64"),
            ))?;
        Ok(geo::Coord::from((origin_x, origin_y)))
    }
    fn get_destination_coordinate(&self) -> Result<Option<geo::Coord<f64>>, PluginError> {
        let x_field = InputField::DestinationX.to_string();
        let y_field = InputField::DestinationY.to_string();
        let x_opt = self.get(&x_field);
        let y_opt = self.get(&y_field);
        match (x_opt, y_opt) {
            (None, None) => Ok(None),
            (None, Some(_)) => Err(PluginError::MissingField(format!(
                "{} provided without {}",
                &y_field, &x_field
            ))),
            (Some(_), None) => Err(PluginError::MissingField(format!(
                "{} provided without {}",
                &x_field, &y_field
            ))),
            (Some(x_json), Some(y_json)) => {
                let x = x_json.as_f64().ok_or(PluginError::ParseError(
                    x_field.clone(),
                    String::from("f64"),
                ))?;
                let y = y_json.as_f64().ok_or(PluginError::ParseError(
                    y_field.clone(),
                    String::from("f64"),
                ))?;
                Ok(Some(geo::Coord::from((x, y))))
            }
        }
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
            _ => Err(PluginError::InputError(String::from(
                "InputQuery is not a JSON object",
            ))),
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
            _ => Err(PluginError::InputError(String::from(
                "InputQuery is not a JSON object",
            ))),
        }
    }

    fn get_origin_vertex(&self) -> Result<VertexId, PluginError> {
        self.get(InputField::OriginVertex.to_string())
            .ok_or(PluginError::MissingField(
                InputField::OriginVertex.to_string(),
            ))?
            .as_u64()
            .map(|v| VertexId(v as usize))
            .ok_or(PluginError::ParseError(
                InputField::OriginVertex.to_string(),
                String::from("u64"),
            ))
    }

    fn get_destination_vertex(&self) -> Result<Option<VertexId>, PluginError> {
        match self.get(InputField::DestinationVertex.to_string()) {
            None => Ok(None),
            Some(v) => {
                v.as_u64()
                    .map(|v| Some(VertexId(v as usize)))
                    .ok_or(PluginError::ParseError(
                        InputField::DestinationVertex.to_string(),
                        String::from("u64"),
                    ))
            }
        }
    }

    fn get_origin_edge(&self) -> Result<EdgeId, PluginError> {
        self.get(InputField::OriginEdge.to_string())
            .ok_or(PluginError::MissingField(
                InputField::OriginEdge.to_string(),
            ))?
            .as_u64()
            .map(|v| EdgeId(v as usize))
            .ok_or(PluginError::ParseError(
                InputField::OriginEdge.to_string(),
                String::from("u64"),
            ))
    }

    fn get_destination_edge(&self) -> Result<Option<EdgeId>, PluginError> {
        match self.get(InputField::DestinationEdge.to_string()) {
            None => Ok(None),
            Some(v) => v
                .as_u64()
                .map(|v| Some(EdgeId(v as usize)))
                .ok_or(PluginError::ParseError(
                    InputField::OriginEdge.to_string(),
                    String::from("u64"),
                )),
        }
    }
    fn get_grid_search(&self) -> Option<&serde_json::Value> {
        self.get(InputField::GridSearch.to_string())
    }

    fn add_origin_edge(&mut self, edge_id: EdgeId) -> Result<(), PluginError> {
        match self {
            serde_json::Value::Object(map) => {
                map.insert(
                    InputField::OriginEdge.to_string(),
                    serde_json::Value::from(edge_id.0),
                );
                Ok(())
            }
            _ => Err(PluginError::InputError(String::from(
                "InputQuery is not a JSON object",
            ))),
        }
    }

    fn add_destination_edge(&mut self, edge_id: EdgeId) -> Result<(), PluginError> {
        match self {
            serde_json::Value::Object(map) => {
                map.insert(
                    InputField::DestinationEdge.to_string(),
                    serde_json::Value::from(edge_id.0),
                );
                Ok(())
            }
            _ => Err(PluginError::InputError(String::from(
                "InputQuery is not a JSON object",
            ))),
        }
    }
}

// pub type DecodeOp<T> = Box<dyn Fn(&serde_json::Value) -> Option<T>>;

// fn get_from_json<T>(
//     value: &serde_json::Value,
//     field: InputField,
//     op: DecodeOp<T>,
// ) -> Result<T, PluginError> {
//     let at_field = value.get(field.to_string());
//     match at_field {
//         None => Err(PluginError::MissingField(field.to_string())),
//         Some(v) => op(v).ok_or(PluginError::ParseError(field.to_string(), ())),
//     };
// }

// fn get_f64(v: &serde_json::Value) -> Result<f64, PluginError> {

//     get_from_json(v, field, |v| v.as_f64())
//     v.as_f64().ok_or(PluginError::ParseError((), ())
// }
