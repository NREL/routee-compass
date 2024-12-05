use super::geometry_model::GeometryModel;
use super::map_error::MapError;
use super::map_model_config::MapModelConfig;
use super::spatial_index::SpatialIndex;
use crate::model::map::map_json_extensions::MapJsonExtensions;
use crate::model::map::nearest_search_result::NearestSearchResult;
use crate::model::network::{EdgeId, Graph};
use geo::LineString;
use std::sync::Arc;

pub struct MapModel {
    pub spatial_index: SpatialIndex,
    pub geometry_model: GeometryModel,
}

impl MapModel {
    pub fn new(graph: Arc<Graph>, config: MapModelConfig) -> Result<MapModel, MapError> {
        match config {
            MapModelConfig::VertexMapModelConfig {
                tolerance,
                geometry_input_file,
            } => {
                let spatial_index =
                    SpatialIndex::new_vertex_oriented(&graph.clone().vertices, tolerance);
                let geometry_model = match geometry_input_file {
                    None => GeometryModel::new_from_vertices(graph),
                    Some(file) => GeometryModel::new_from_edges(&file, graph.clone()),
                }?;
                let map_model = MapModel {
                    spatial_index,
                    geometry_model,
                };
                Ok(map_model)
            }
            MapModelConfig::EdgeMapModelConfig {
                tolerance,
                geometry_input_file,
            } => {
                let geometry_model =
                    GeometryModel::new_from_edges(&geometry_input_file, graph.clone())?;
                let spatial_index =
                    SpatialIndex::new_edge_oriented(graph.clone(), &geometry_model, tolerance);
                let map_model = MapModel {
                    spatial_index,
                    geometry_model,
                };
                Ok(map_model)
            }
        }
    }

    pub fn get<'a>(&'a self, edge_id: &EdgeId) -> Result<&'a LineString<f32>, MapError> {
        self.geometry_model.get(edge_id)
    }

    pub fn map_match(&self, query: &mut serde_json::Value) -> Result<(), MapError> {
        let src_point = geo::Point(query.get_origin_coordinate()?);
        match self.spatial_index.nearest_graph_id(&src_point)? {
            NearestSearchResult::NearestVertex(vertex_id) => {
                query.add_origin_vertex(vertex_id)?;
            }
            NearestSearchResult::NearestEdge(edge_id) => query.add_origin_edge(edge_id)?,
        }

        let dst_coord_option = query.get_destination_coordinate()?;
        match dst_coord_option {
            None => {}
            Some(dst_coord) => {
                let dst_point = geo::Point(dst_coord);
                match self.spatial_index.nearest_graph_id(&dst_point)? {
                    NearestSearchResult::NearestVertex(vertex_id) => {
                        query.add_destination_vertex(vertex_id)?;
                    }
                    NearestSearchResult::NearestEdge(edge_id) => {
                        query.add_destination_edge(edge_id)?
                    }
                }
            }
        }

        Ok(())
    }
}
