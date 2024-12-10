use super::{
    map_error::MapError, map_json_extensions::MapJsonExtensions, map_model::MapModel,
    nearest_search_result::NearestSearchResult,
};
use crate::model::{
    frontier::frontier_model::FrontierModel,
    network::{Edge, Graph},
};
use itertools::Itertools;
use serde::{de, Deserialize, Deserializer, Serialize};
use std::{fmt::Display, sync::Arc};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum MatchingType {
    /// expect origin [, destination] VertexIds on the query.
    VertexId,
    /// expect origin [, destination] EdgeIds on the query.
    EdgeId,
    /// expect origin [, destination] Points on the query.
    Point,
    /// expect any combination of the map input types provided
    #[serde(deserialize_with = "de_combined")]
    Combined(Vec<MatchingType>),
}

impl Default for MatchingType {
    /// the default MatchingType is to first attempt to process a Point into VertexIds,
    /// then attempt to find VertexIds on the query,
    /// then finally attempt to find EdgeIds on the query.
    fn default() -> Self {
        Self::Combined(vec![Self::Point, Self::VertexId, Self::EdgeId])
    }
}

impl Display for MatchingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let self_str = match self {
            MatchingType::Combined(vec) => vec.iter().map(|mit| mit.to_string()).join(","),
            MatchingType::VertexId => String::from("vertex_id"),
            MatchingType::EdgeId => String::from("edge_id"),
            MatchingType::Point => String::from("point"),
        };
        write!(f, "{}", self_str)
    }
}

pub enum MapInputResult {
    Found,
    NotFound,
}

impl MatchingType {
    /// attempts to find any valid input origin fields, or performs map matching, in order to
    /// append those fields to the query, based on the type of [`MatchingType`] supported.
    pub fn process_origin(
        &self,
        map_model: &MapModel,
        frontier_model: Arc<dyn FrontierModel>,
        graph: Arc<Graph>,
        query: &mut serde_json::Value,
    ) -> Result<(), MapError> {
        use MatchingType as MIT;
        match self {
            MIT::Combined(vec) => {
                let mut errors = vec![];
                for matching_type in vec.iter() {
                    match matching_type.process_origin(
                        map_model,
                        frontier_model.clone(),
                        graph.clone(),
                        query,
                    ) {
                        Ok(_) => return Ok(()),
                        Err(e) => {
                            let mit = serde_json::to_string(matching_type).unwrap_or_default();
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
            MIT::VertexId => {
                // validate all out-edges for this vertex
                let vertex_id = query.get_origin_vertex()?;
                let edges = graph.out_edges(&vertex_id).iter().map(|edge_id| graph.get_edge(edge_id)).collect::<Result<Vec<_>, _>>().map_err(|e| MapError::MapMatchError(format!("while attempting to validate vertex id {} for map matching, the underlying Graph model caused an error: {}", vertex_id, e)))?;
                for edge in edges.into_iter() {
                    if let Ok(true) = test_edge(edge, frontier_model.clone()) {
                        return Ok(());
                    }
                }
                Err(MapError::MapMatchError(format!("attempted to map match origin vertex_id {} provided in query, but no out-edges are valid for traversal according to this FrontierModel instance", vertex_id)))
            }
            MIT::EdgeId => {
                // validate this edge
                let edge_id = query.get_origin_edge()?;
                let edge = graph.get_edge(&edge_id).map_err(|e| MapError::MapMatchError(format!("while attempting to validate edge id {} for map matching, the underlying Graph model caused an error: {}", edge_id, e)))?;
                validate_edge(edge, frontier_model.clone())
            }
            MIT::Point => {
                // iterate through nearest values in the spatial index to this point that
                // are within our matching tolerance and validate them with the frontier model
                let src_point = geo::Point(query.get_origin_coordinate()?);
                for nearest in map_model.spatial_index.nearest_graph_id_iter(&src_point) {
                    match nearest {
                        NearestSearchResult::NearestVertex(vertex_id) => {
                            // if any of the out-edges of this vertex are valid, we can finish
                            let edges = graph.out_edges(&vertex_id).iter().map(|edge_id| graph.get_edge(edge_id)).collect::<Result<Vec<_>, _>>().map_err(|e| MapError::MapMatchError(format!("while attempting to validate vertex id {} for map matching, the underlying Graph model caused an error: {}", vertex_id, e)))?;
                            for edge in edges.into_iter() {
                                let is_valid = test_edge(edge, frontier_model.clone())?;
                                if is_valid {
                                    query.add_origin_vertex(vertex_id)?;
                                    return Ok(());
                                }
                            }
                            continue;
                        }
                        NearestSearchResult::NearestEdge(edge_id) => {
                            let edge = graph.get_edge(&edge_id).map_err(|e| MapError::MapMatchError(format!("while attempting to validate edge id {} from nearest neighbor search for map matching, the underlying Graph model caused an error: {}", edge_id, e)))?;
                            let is_valid = test_edge(edge, frontier_model.clone())?;
                            if is_valid {
                                query.add_origin_edge(edge_id)?;
                                return Ok(());
                            }
                        }
                    }
                }
                Err(MapError::MapMatchError(format!(
                    "attempted to match query origin coordinate ({}, {}) to map but exausted all possibilities",
                    src_point.x(),
                    src_point.y(),
                )))
            }
        }
    }

    /// attempts to find any valid input destination fields, or performs map matching, in order to
    /// append those fields to the query, based on the type of [`MatchingType`] supported.
    /// since destinations are optional, the method returns a [`MapInputResult`] that indicates if
    /// a destination was found or not found.
    pub fn process_destination(
        &self,
        map_model: &MapModel,
        frontier_model: Arc<dyn FrontierModel>,
        graph: Arc<Graph>,
        query: &mut serde_json::Value,
    ) -> Result<MapInputResult, MapError> {
        use MatchingType as MIT;
        match self {
            MIT::Combined(vec) => {
                let mut errors = vec![];
                for matching_type in vec.iter() {
                    match matching_type.process_destination(
                        map_model,
                        frontier_model.clone(),
                        graph.clone(),
                        query,
                    ) {
                        Ok(_) => return Ok(MapInputResult::Found),
                        Err(e) => {
                            let mit = serde_json::to_string(matching_type).unwrap_or_default();
                            let msg = format!("no destination {} on input query: {}", mit, e);
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

            MIT::VertexId => {
                // validate all out-edges for this vertex, if one is accepted, we are done.
                let vertex_id_option = query.get_destination_vertex()?;
                match vertex_id_option {
                    Some(vertex_id) => {
                        let edges = graph.in_edges(&vertex_id).iter().map(|edge_id| graph.get_edge(edge_id)).collect::<Result<Vec<_>, _>>().map_err(|e| MapError::MapMatchError(format!("while attempting to validate vertex id {} for map matching, the underlying Graph model caused an error: {}", vertex_id, e)))?;
                        for edge in edges.into_iter() {
                            if let Ok(true) = test_edge(edge, frontier_model.clone()) {
                                return Ok(MapInputResult::Found);
                            }
                        }
                        Err(MapError::MapMatchError(format!("attempted to map match destination vertex_id {} provided in query, but no in-edges are valid for traversal according to this FrontierModel instance", vertex_id)))
                    }
                    None => Ok(MapInputResult::NotFound),
                }
            }

            MIT::EdgeId => {
                // validate this edge
                let dest_edge_option = query.get_destination_edge()?;
                match dest_edge_option {
                    Some(edge_id) => {
                        let edge = graph.get_edge(&edge_id).map_err(|e| MapError::MapMatchError(format!("while attempting to validate edge id {} for map matching, the underlying Graph model caused an error: {}", edge_id, e)))?;
                        validate_edge(edge, frontier_model.clone())?;
                        Ok(MapInputResult::Found)
                    }
                    None => Ok(MapInputResult::NotFound),
                }
            }

            MIT::Point => {
                // iterate through nearest values in the spatial index to this point that
                // are within our matching tolerance and validate them with the frontier model
                let dst_point = match query.get_destination_coordinate()? {
                    Some(coord) => geo::Point(coord),
                    None => return Ok(MapInputResult::NotFound),
                };

                for nearest in map_model.spatial_index.nearest_graph_id_iter(&dst_point) {
                    match nearest {
                        NearestSearchResult::NearestVertex(vertex_id) => {
                            // if any of the out-edges of this vertex are valid, we can finish
                            let edges = graph.out_edges(&vertex_id).iter().map(|edge_id| graph.get_edge(edge_id)).collect::<Result<Vec<_>, _>>().map_err(|e| MapError::MapMatchError(format!("while attempting to validate vertex id {} for map matching, the underlying Graph model caused an error: {}", vertex_id, e)))?;
                            for edge in edges.into_iter() {
                                let is_valid = test_edge(edge, frontier_model.clone())?;
                                if is_valid {
                                    query.add_destination_vertex(vertex_id)?;
                                    return Ok(MapInputResult::Found);
                                }
                            }
                            continue;
                        }
                        NearestSearchResult::NearestEdge(edge_id) => {
                            let edge = graph.get_edge(&edge_id).map_err(|e| MapError::MapMatchError(format!("while attempting to validate edge id {} from nearest neighbor search for map matching, the underlying Graph model caused an error: {}", edge_id, e)))?;
                            let is_valid = test_edge(edge, frontier_model.clone())?;
                            if is_valid {
                                query.add_destination_edge(edge_id)?;
                                return Ok(MapInputResult::Found);
                            }
                        }
                    }
                }
                Err(MapError::MapMatchError(format!(
                    "attempted to match query destination coordinate ({}, {}) to map but exausted all possibilities",
                    dst_point.x(),
                    dst_point.y(),
                )))
            }
        }
    }
}

fn test_edge(edge: &Edge, fm: Arc<dyn FrontierModel>) -> Result<bool, MapError> {
    let is_valid = fm.valid_edge(edge).map_err(|e| MapError::MapMatchError(format!("while attempting to validate edge id {} for map matching, the underlying FrontierModel caused an error: {}", edge.edge_id, e)))?;
    Ok(is_valid)
}

fn validate_edge(edge: &Edge, fm: Arc<dyn FrontierModel>) -> Result<(), MapError> {
    let is_valid = test_edge(edge, fm)?;
    if !is_valid {
        Err(MapError::MapMatchError(format!(
            "query assigned origin of edge {} is not valid according to the FrontierModel",
            edge.edge_id
        )))
    } else {
        Ok(())
    }
}

fn de_combined<'de, D>(value: D) -> Result<Vec<MatchingType>, D::Error>
where
    D: Deserializer<'de>,
{
    struct CombinedVisitor;

    impl<'de> de::Visitor<'de> for CombinedVisitor {
        type Value = Vec<MatchingType>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a vector of MatchingType strings")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            let mut out: Vec<MatchingType> = vec![];
            while let Some(next) = seq.next_element()? {
                if let MatchingType::Combined(_) = next {
                    return Err(serde::de::Error::custom(String::from(
                        "cannot deeply nest matching_type entries",
                    )));
                }
                out.push(next);
            }
            Ok(out)
        }
    }

    value.deserialize_seq(CombinedVisitor {})
}
