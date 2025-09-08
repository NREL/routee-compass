use super::{map_error::MapError, map_json_key::MapJsonKey};
use crate::model::network::{EdgeId, EdgeListId, VertexId};
use geo;

pub trait MapJsonExtensions {
    fn get_origin_coordinate(&self) -> Result<geo::Coord<f32>, MapError>;
    fn get_destination_coordinate(&self) -> Result<Option<geo::Coord<f32>>, MapError>;
    fn add_origin_vertex(&mut self, vertex_id: VertexId) -> Result<(), MapError>;
    fn add_destination_vertex(&mut self, vertex_id: VertexId) -> Result<(), MapError>;
    fn add_origin_edge(
        &mut self,
        edge_list_id: EdgeListId,
        edge_id: EdgeId,
    ) -> Result<(), MapError>;
    fn add_destination_edge(
        &mut self,
        edge_list_id: EdgeListId,
        edge_id: EdgeId,
    ) -> Result<(), MapError>;
    fn get_origin_vertex(&self) -> Result<VertexId, MapError>;
    fn get_destination_vertex(&self) -> Result<Option<VertexId>, MapError>;
    fn get_origin_edge(&self) -> Result<(EdgeListId, EdgeId), MapError>;
    fn get_destination_edge(&self) -> Result<Option<(EdgeListId, EdgeId)>, MapError>;
}

impl MapJsonExtensions for serde_json::Value {
    fn get_origin_coordinate(&self) -> Result<geo::Coord<f32>, MapError> {
        let origin_x = self
            .get(MapJsonKey::OriginX.to_string())
            .ok_or(MapError::InputMissingField(MapJsonKey::OriginX))?
            .as_f64()
            .ok_or_else(|| {
                MapError::InputDeserializingError(
                    MapJsonKey::OriginX.to_string(),
                    String::from("f64"),
                )
            })?;
        let origin_y = self
            .get(MapJsonKey::OriginY.to_string())
            .ok_or(MapError::InputMissingField(MapJsonKey::OriginY))?
            .as_f64()
            .ok_or_else(|| {
                MapError::InputDeserializingError(
                    MapJsonKey::OriginY.to_string(),
                    String::from("f64"),
                )
            })?;
        Ok(geo::Coord::from((origin_x as f32, origin_y as f32)))
    }
    fn get_destination_coordinate(&self) -> Result<Option<geo::Coord<f32>>, MapError> {
        let x_field = MapJsonKey::DestinationX;
        let y_field = MapJsonKey::DestinationY;
        let x_opt = self.get(x_field.to_string());
        let y_opt = self.get(y_field.to_string());
        match (x_opt, y_opt) {
            (None, None) => Ok(None),
            (None, Some(_)) => Err(MapError::InputMissingPairedField(y_field, x_field)),
            (Some(_), None) => Err(MapError::InputMissingPairedField(x_field, y_field)),
            (Some(x_json), Some(y_json)) => {
                let x = x_json.as_f64().ok_or_else(|| {
                    MapError::InputDeserializingError(x_field.to_string(), String::from("f64"))
                })?;
                let y = y_json.as_f64().ok_or_else(|| {
                    MapError::InputDeserializingError(y_field.to_string(), String::from("f64"))
                })?;
                Ok(Some(geo::Coord::from((x as f32, y as f32))))
            }
        }
    }
    fn add_origin_vertex(&mut self, vertex_id: VertexId) -> Result<(), MapError> {
        match self {
            serde_json::Value::Object(map) => {
                map.insert(
                    MapJsonKey::OriginVertex.to_string(),
                    serde_json::Value::from(vertex_id.0),
                );
                Ok(())
            }
            _ => Err(MapError::InputDeserializingError(
                String::from("<user query>"),
                String::from("json object"),
            )),
        }
    }
    fn add_destination_vertex(&mut self, vertex_id: VertexId) -> Result<(), MapError> {
        match self {
            serde_json::Value::Object(map) => {
                map.insert(
                    MapJsonKey::DestinationVertex.to_string(),
                    serde_json::Value::from(vertex_id.0),
                );
                Ok(())
            }
            _ => Err(MapError::InputDeserializingError(
                String::from("<user query>"),
                String::from("json object"),
            )),
        }
    }

    fn get_origin_vertex(&self) -> Result<VertexId, MapError> {
        let key = MapJsonKey::OriginVertex.to_string();
        self.get(&key)
            .ok_or(MapError::InputMissingField(MapJsonKey::OriginVertex))?
            .as_u64()
            .map(|v| VertexId(v as usize))
            .ok_or_else(|| MapError::InputDeserializingError(key, String::from("u64")))
    }

    fn get_destination_vertex(&self) -> Result<Option<VertexId>, MapError> {
        let key = MapJsonKey::DestinationVertex.to_string();
        match self.get(&key) {
            None => Ok(None),
            Some(v) => v
                .as_u64()
                .map(|v| Some(VertexId(v as usize)))
                .ok_or_else(|| MapError::InputDeserializingError(key, String::from("u64"))),
        }
    }

    fn get_origin_edge(&self) -> Result<(EdgeListId, EdgeId), MapError> {
        let edge_list_id = self
            .get(MapJsonKey::OriginEdgeList.as_str())
            .ok_or(MapError::InputMissingField(MapJsonKey::OriginEdgeList))?
            .as_u64()
            .map(|v| EdgeListId(v as usize))
            .ok_or_else(|| {
                MapError::InputDeserializingError(
                    MapJsonKey::OriginEdgeList.to_string(),
                    String::from("u64"),
                )
            })?;
        let edge_id = self
            .get(MapJsonKey::OriginEdge.as_str())
            .ok_or(MapError::InputMissingField(MapJsonKey::OriginEdge))?
            .as_u64()
            .map(|v| EdgeId(v as usize))
            .ok_or_else(|| {
                MapError::InputDeserializingError(
                    MapJsonKey::OriginEdge.to_string(),
                    String::from("u64"),
                )
            })?;
        Ok((edge_list_id, edge_id))
    }

    fn get_destination_edge(&self) -> Result<Option<(EdgeListId, EdgeId)>, MapError> {
        let lookup = (
            self.get(MapJsonKey::DestinationEdgeList.as_str()),
            self.get(MapJsonKey::DestinationEdge.as_str()),
        );
        match lookup {
            (None, None) => Ok(None),
            (None, Some(_)) => Err(MapError::InputMissingPairedField(
                MapJsonKey::DestinationEdge,
                MapJsonKey::DestinationEdgeList,
            )),
            (Some(_), None) => Err(MapError::InputMissingPairedField(
                MapJsonKey::DestinationEdgeList,
                MapJsonKey::DestinationEdge,
            )),
            (Some(edge_list_json), Some(edge_json)) => {
                let edge_list_id = edge_list_json
                    .as_u64()
                    .map(|v| EdgeListId(v as usize))
                    .ok_or_else(|| {
                        MapError::InputDeserializingError(
                            MapJsonKey::DestinationEdgeList.to_string(),
                            String::from("u64"),
                        )
                    })?;
                let edge_id = edge_json
                    .as_u64()
                    .map(|v| EdgeId(v as usize))
                    .ok_or_else(|| {
                        MapError::InputDeserializingError(
                            MapJsonKey::DestinationEdge.to_string(),
                            String::from("u64"),
                        )
                    })?;
                Ok(Some((edge_list_id, edge_id)))
            }
        }
    }

    fn add_origin_edge(
        &mut self,
        edge_list_id: EdgeListId,
        edge_id: EdgeId,
    ) -> Result<(), MapError> {
        match self {
            serde_json::Value::Object(map) => {
                map.insert(
                    MapJsonKey::OriginEdgeList.to_string(),
                    serde_json::Value::from(edge_list_id.0),
                );
                map.insert(
                    MapJsonKey::OriginEdge.to_string(),
                    serde_json::Value::from(edge_id.0),
                );
                Ok(())
            }
            _ => Err(MapError::InputDeserializingError(
                String::from("<user query>"),
                String::from("json object"),
            )),
        }
    }

    fn add_destination_edge(
        &mut self,
        edge_list_id: EdgeListId,
        edge_id: EdgeId,
    ) -> Result<(), MapError> {
        match self {
            serde_json::Value::Object(map) => {
                map.insert(
                    MapJsonKey::DestinationEdgeList.to_string(),
                    serde_json::Value::from(edge_list_id.0),
                );
                map.insert(
                    MapJsonKey::DestinationEdge.to_string(),
                    serde_json::Value::from(edge_id.0),
                );
                Ok(())
            }
            _ => Err(MapError::InputDeserializingError(
                String::from("<user query>"),
                String::from("json object"),
            )),
        }
    }
}
