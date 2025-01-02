use super::mapping_app_error::MappingAppError;
use geo::LineString;
use routee_compass_core::model::{map::MapModel, network::EdgeId};

/// stub for a binary centered on map matching
pub struct MappingApp {
    pub map_model: MapModel,
}

impl MappingApp {
    // pub fn append_graph_ids(&self, query: &mut serde_json::Value) -> Result<(), MappingAppError> {
    //     self.map_model
    //         .map_match(query)
    //         .map_err(MappingAppError::MapError)
    // }

    pub fn get_edge_linestring(
        &self,
        edge_id: EdgeId,
    ) -> Result<&LineString<f32>, MappingAppError> {
        self.map_model
            .get(&edge_id)
            .map_err(MappingAppError::MapError)
    }
}
