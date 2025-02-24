use std::borrow::Cow;

use super::search_app::SearchApp;
use crate::app::compass::CompassAppError;
use routee_compass_core::{
    algorithm::search::Direction,
    model::{
        network::{edge_id::EdgeId, vertex_id::VertexId},
        unit::{Convert, Distance, DistanceUnit},
    },
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
            Some(du) => {
                let mut dist_convert = Cow::Owned(result_base);
                DistanceUnit::Meters.convert(&mut dist_convert, &du).map_err(|e| CompassAppError::InternalError(format!("while getting an edge distance, the internal units conversion library failed with: {}", e)))?;
                dist_convert.into_owned()
            }
            None => result_base,
        };
        Ok(result)
    }

    fn get_incident_edge_ids(&self, vertex_id: &VertexId, direction: &Direction) -> Vec<EdgeId> {
        self.graph.incident_edges(vertex_id, direction)
    }
}
