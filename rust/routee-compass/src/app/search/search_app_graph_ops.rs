use super::search_app::SearchApp;
use crate::app::compass::CompassAppError;
use routee_compass_core::{
    algorithm::search::Direction,
    model::network::{EdgeId, EdgeListId, VertexId},
};
use uom::si::f64::Length;

pub trait SearchAppGraphOps {
    fn get_edge_origin(
        &self,
        edge_list_id: &EdgeListId,
        edge_id: &EdgeId,
    ) -> Result<VertexId, CompassAppError>;
    fn get_edge_destination(
        &self,
        edge_list_id: &EdgeListId,
        edge_id: &EdgeId,
    ) -> Result<VertexId, CompassAppError>;
    fn get_edge_distance(
        &self,
        edge_list_id: &EdgeListId,
        edge_id: &EdgeId,
    ) -> Result<Length, CompassAppError>;
    fn get_incident_edge_ids(
        &self,
        vertex_id: &VertexId,
        direction: &Direction,
    ) -> Vec<(EdgeListId, EdgeId)>;
}

impl SearchAppGraphOps for SearchApp {
    fn get_edge_origin(
        &self,
        edge_list_id: &EdgeListId,
        edge_id: &EdgeId,
    ) -> Result<VertexId, CompassAppError> {
        let edge = self.graph.get_edge(edge_list_id, edge_id)?;
        Ok(edge.src_vertex_id)
    }

    fn get_edge_destination(
        &self,
        edge_list_id: &EdgeListId,
        edge_id: &EdgeId,
    ) -> Result<VertexId, CompassAppError> {
        let edge = self.graph.get_edge(edge_list_id, edge_id)?;
        Ok(edge.dst_vertex_id)
    }

    fn get_edge_distance(
        &self,
        edge_list_id: &EdgeListId,
        edge_id: &EdgeId,
    ) -> Result<Length, CompassAppError> {
        let edge = self.graph.get_edge(edge_list_id, edge_id)?;
        Ok(edge.distance)
    }

    fn get_incident_edge_ids(
        &self,
        vertex_id: &VertexId,
        direction: &Direction,
    ) -> Vec<(EdgeListId, EdgeId)> {
        self.graph.incident_edges(vertex_id, direction)
    }
}
