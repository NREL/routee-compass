use super::map_error::MapError;
use super::map_input_type::MapInputType;
use super::map_model_config::MapModelConfig;
use super::spatial_index::SpatialIndex;
use super::{geometry_model::GeometryModel, map_input_type::MapInputResult};
use crate::model::network::{EdgeId, Graph};
use geo::LineString;
use std::sync::Arc;

pub struct MapModel {
    pub map_input_type: MapInputType,
    pub spatial_index: SpatialIndex,
    pub geometry_model: GeometryModel,
    pub queries_without_destinations: bool,
}

impl MapModel {
    pub fn new(graph: Arc<Graph>, config: MapModelConfig) -> Result<MapModel, MapError> {
        match config {
            MapModelConfig::VertexMapModelConfig {
                map_input_type,
                tolerance,
                geometry_input_file,
                queries_without_destinations,
            } => {
                let spatial_index =
                    SpatialIndex::new_vertex_oriented(&graph.clone().vertices, tolerance);
                let geometry_model = match geometry_input_file {
                    None => GeometryModel::new_from_vertices(graph),
                    Some(file) => GeometryModel::new_from_edges(&file, graph.clone()),
                }?;
                let map_model = MapModel {
                    map_input_type,
                    spatial_index,
                    geometry_model,
                    queries_without_destinations,
                };
                Ok(map_model)
            }
            MapModelConfig::EdgeMapModelConfig {
                map_input_type,
                tolerance,
                geometry_input_file,
                queries_without_destinations,
            } => {
                let geometry_model =
                    GeometryModel::new_from_edges(&geometry_input_file, graph.clone())?;
                let spatial_index =
                    SpatialIndex::new_edge_oriented(graph.clone(), &geometry_model, tolerance);
                let map_model = MapModel {
                    map_input_type,
                    spatial_index,
                    geometry_model,
                    queries_without_destinations,
                };
                Ok(map_model)
            }
        }
    }

    pub fn get<'a>(&'a self, edge_id: &EdgeId) -> Result<&'a LineString<f32>, MapError> {
        self.geometry_model.get(edge_id)
    }

    pub fn map_match(&self, query: &mut serde_json::Value) -> Result<(), MapError> {
        self.map_input_type.process_origin(self, query)?;
        match self.map_input_type.process_destination(self, query)? {
            MapInputResult::NotFound if !self.queries_without_destinations => {
                Err(MapError::DestinationsRequired(self.map_input_type.clone()))
            }
            _ => Ok(()),
        }
    }
}
