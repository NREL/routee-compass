use super::{
    map_error::MapError, map_json_extensions::MapJsonExtensions, map_model::MapModel,
    nearest_search_result::NearestSearchResult,
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum MapInputType {
    /// expect any combination of the map input types provided
    Combined(Vec<Box<MapInputType>>),
    /// expect origin [, destination] VertexIds on the query.
    VertexId,
    /// expect origin [, destination] EdgeIds on the query.
    EdgeId,
    /// expect origin [, destination] Points on the query.
    Point,
}

impl Default for MapInputType {
    /// the default MapInputType is to first attempt to process a Point into VertexIds,
    /// then attempt to find VertexIds on the query,
    /// then finally attempt to find EdgeIds on the query.
    fn default() -> Self {
        Self::Combined(vec![
            Box::new(Self::Point),
            Box::new(Self::VertexId),
            Box::new(Self::EdgeId),
        ])
    }
}

impl Display for MapInputType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let self_str = match self {
            MapInputType::Combined(vec) => vec.iter().map(|mit| mit.to_string()).join(","),
            MapInputType::VertexId => String::from("vertex_id"),
            MapInputType::EdgeId => String::from("edge_id"),
            MapInputType::Point => String::from("point"),
        };
        write!(f, "{}", self_str)
    }
}

pub enum MapInputResult {
    Found,
    NotFound,
}

impl MapInputType {
    /// attempts to find any valid input origin fields, or performs map matching, in order to
    /// append those fields to the query, based on the type of [`MapInputType`] supported.
    pub fn process_origin(
        &self,
        map_model: &MapModel,
        query: &mut serde_json::Value,
    ) -> Result<(), MapError> {
        // process origin
        match self {
            MapInputType::Combined(vec) => {
                let mut errors = vec![];
                for map_input_type in vec.iter() {
                    match map_input_type.process_origin(map_model, query) {
                        Ok(_) => return Ok(()),
                        Err(e) => {
                            let mit = serde_json::to_string(map_input_type).unwrap_or_default();
                            let msg = format!("no origin {} on input query: {}", mit, e);
                            errors.push(msg);
                        }
                    }
                }
                if !errors.is_empty() {
                    let msg = errors.iter().join("; ");
                    Err(MapError::MapMatchError(format!(
                        "unable to match query to map: {}",
                        msg
                    )))
                } else {
                    Ok(())
                }
            }
            MapInputType::VertexId => query.get_origin_vertex().map(|_| ()),
            MapInputType::EdgeId => query.get_origin_edge().map(|_| ()),
            MapInputType::Point => {
                let src_point = geo::Point(query.get_origin_coordinate()?);
                match map_model.spatial_index.nearest_graph_id(&src_point)? {
                    NearestSearchResult::NearestVertex(vertex_id) => {
                        query.add_origin_vertex(vertex_id)?;
                    }
                    NearestSearchResult::NearestEdge(edge_id) => query.add_origin_edge(edge_id)?,
                }

                Ok(())
            }
        }
    }

    /// attempts to find any valid input destination fields, or performs map matching, in order to
    /// append those fields to the query, based on the type of [`MapInputType`] supported.
    /// since destinations are optional, the method returns a [`MapInputResult`] that indicates if
    /// a destination was found or not found.
    pub fn process_destination(
        &self,
        map_model: &MapModel,
        query: &mut serde_json::Value,
    ) -> Result<MapInputResult, MapError> {
        match self {
            MapInputType::Combined(vec) => {
                let mut errors = vec![];
                for map_input_type in vec.iter() {
                    match map_input_type.process_destination(map_model, query) {
                        Ok(_) => return Ok(MapInputResult::Found),
                        Err(e) => {
                            let mit = serde_json::to_string(map_input_type).unwrap_or_default();
                            let msg = format!("no origin {} on input query: {}", mit, e);
                            errors.push(msg);
                        }
                    }
                }
                if !errors.is_empty() {
                    let msg = errors.iter().join("; ");
                    Err(MapError::MapMatchError(format!(
                        "unable to match query to map: {}",
                        msg
                    )))
                } else {
                    Ok(MapInputResult::NotFound)
                }
            }
            MapInputType::VertexId => {
                let dest_vertex_id_result = query.get_destination_vertex();
                dest_vertex_id_result.map(|result| match result {
                    Some(_) => MapInputResult::Found,
                    None => MapInputResult::NotFound,
                })
            }
            MapInputType::EdgeId => {
                let dest_edge_id_result = query.get_destination_edge();
                dest_edge_id_result.map(|result| match result {
                    Some(_) => MapInputResult::Found,
                    None => MapInputResult::NotFound,
                })
            }
            MapInputType::Point => {
                let dst_coord_option = query.get_destination_coordinate()?;
                match dst_coord_option {
                    None => Ok(MapInputResult::NotFound),
                    Some(dst_coord) => {
                        let dst_point = geo::Point(dst_coord);
                        match map_model.spatial_index.nearest_graph_id(&dst_point)? {
                            NearestSearchResult::NearestVertex(vertex_id) => {
                                query.add_destination_vertex(vertex_id)?;
                            }
                            NearestSearchResult::NearestEdge(edge_id) => {
                                query.add_destination_edge(edge_id)?;
                            }
                        }
                        Ok(MapInputResult::Found)
                    }
                }
            }
        }
    }
}
