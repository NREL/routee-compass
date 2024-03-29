use super::search_app::SearchApp;
use crate::app::compass::compass_app_error::CompassAppError;
use routee_compass_core::{
    algorithm::search::direction::Direction,
    model::road_network::{edge_id::EdgeId, vertex_id::VertexId},
    model::unit::{Distance, DistanceUnit},
};

pub trait SearchAppGraphOps {
    fn get_edge_origin(&self, edge_id: EdgeId) -> Result<VertexId, CompassAppError>;
    fn get_edge_destination(&self, edge_id: EdgeId) -> Result<VertexId, CompassAppError>;
    fn get_edge_distance(
        &self,
        edge_id: EdgeId,
        distance_unit: Option<DistanceUnit>,
    ) -> Result<Distance, CompassAppError>;
    fn get_incident_edge_ids(
        &self,
        vertex_id: VertexId,
        direction: Direction,
    ) -> Result<Vec<EdgeId>, CompassAppError>;
}

impl SearchAppGraphOps for SearchApp {
    fn get_edge_origin(&self, edge_id: EdgeId) -> Result<VertexId, CompassAppError> {
        let edge = self
            .directed_graph
            .get_edge(edge_id)
            .map_err(CompassAppError::GraphError)?;
        Ok(edge.src_vertex_id)
    }

    fn get_edge_destination(&self, edge_id: EdgeId) -> Result<VertexId, CompassAppError> {
        let edge = self
            .directed_graph
            .get_edge(edge_id)
            .map_err(CompassAppError::GraphError)?;
        Ok(edge.dst_vertex_id)
    }

    fn get_edge_distance(
        &self,
        edge_id: EdgeId,
        distance_unit: Option<DistanceUnit>,
    ) -> Result<Distance, CompassAppError> {
        let edge = self
            .directed_graph
            .get_edge(edge_id)
            .map_err(CompassAppError::GraphError)?;
        let result_base = edge.distance;
        let result = match distance_unit {
            Some(du) => DistanceUnit::Meters.convert(&result_base, &du),
            None => result_base,
        };
        Ok(result)
    }

    fn get_incident_edge_ids(
        &self,
        vertex_id: VertexId,
        direction: Direction,
    ) -> Result<Vec<EdgeId>, CompassAppError> {
        let incident_edges = self
            .directed_graph
            .incident_edges(vertex_id, direction)
            .map_err(CompassAppError::GraphError)?;
        Ok(incident_edges)
    }
}
