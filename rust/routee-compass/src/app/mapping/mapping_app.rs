use super::mapping_app_error::MappingAppError;
use geo::LineString;
use routee_compass_core::model::{map::map_model::MapModel, road_network::edge_id::EdgeId};
use std::sync::Arc;

pub struct MappingApp<'a> {
    pub map_model: Arc<MapModel<'a>>,
    pub edge_geometries: Vec<LineString>,
}

impl<'a> MappingApp<'a> {
    pub fn append_graph_ids(&self, query: &mut serde_json::Value) -> Result<(), MappingAppError> {
        self.map_model.map_match(query).map_err(|e| e.into())
    }

    pub fn get_edge_wkt(&'a self, edge_id: EdgeId) -> Result<&'a LineString, MappingAppError> {
        self.edge_geometries
            .get(edge_id.0)
            .ok_or(MappingAppError::InvalidEdgeId(edge_id))
    }
}
