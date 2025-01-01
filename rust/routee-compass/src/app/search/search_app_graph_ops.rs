use super::search_app::SearchApp;
use crate::app::compass::CompassAppError;
use routee_compass_core::{
    algorithm::search::Direction,
    model::network::{edge_id::EdgeId, vertex_id::VertexId},
    model::unit::{Distance, DistanceUnit},
};

pub trait SearchAppGraphOps {
    fn get_edge_origin(&self, edge_id: &EdgeId) -> Result<VertexId, CompassAppError>;
    fn get_edge_destination(&self, edge_id: &EdgeId) -> Result<VertexId, CompassAppError>;
    fn get_edge_distance(
        &self,
        edge_id: &EdgeId,
        distance_unit: Option<DistanceUnit>,
    ) -> Result<Distance, CompassAppError>;
    fn get_incident_edge_ids(&self, vertex_id: &VertexId, direction: &Direction) -> Vec<EdgeId>;
}

impl SearchAppGraphOps for SearchApp {
    fn get_edge_origin(&self, edge_id: &EdgeId) -> Result<VertexId, CompassAppError> {
        let edge = self.graph.get_edge(edge_id)?;
        Ok(edge.src_vertex_id)
    }

    fn get_edge_destination(&self, edge_id: &EdgeId) -> Result<VertexId, CompassAppError> {
        let edge = self.graph.get_edge(edge_id)?;
        Ok(edge.dst_vertex_id)
    }

    fn get_edge_distance(
        &self,
        edge_id: &EdgeId,
        distance_unit: Option<DistanceUnit>,
    ) -> Result<Distance, CompassAppError> {
        let edge = self.graph.get_edge(edge_id)?;
        let result_base = edge.distance;
        let result = match distance_unit {
            Some(du) => DistanceUnit::Meters.convert(&result_base, &du),
            None => result_base,
        };
        Ok(result)
    }

    fn get_incident_edge_ids(&self, vertex_id: &VertexId, direction: &Direction) -> Vec<EdgeId> {
        self.graph.incident_edges(vertex_id, direction)
    }
}
