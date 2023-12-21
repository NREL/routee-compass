use routee_compass_core::{
    algorithm::search::direction::Direction,
    model::road_network::{edge_id::EdgeId, graph::Graph, vertex_id::VertexId},
    model::unit::{Distance, DistanceUnit},
};

use crate::app::compass::compass_app_error::CompassAppError;

use super::search_app::SearchApp;

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
        let op = move |g: &Graph| {
            let edge = g.get_edge(edge_id).map_err(CompassAppError::GraphError)?;
            Ok(edge.src_vertex_id)
        };
        let result: VertexId = graph_op(self, &op)?;
        Ok(result)
    }

    fn get_edge_destination(&self, edge_id: EdgeId) -> Result<VertexId, CompassAppError> {
        let op = move |g: &Graph| {
            let edge = g.get_edge(edge_id).map_err(CompassAppError::GraphError)?;
            Ok(edge.dst_vertex_id)
        };
        let result: VertexId = graph_op(self, &op)?;
        Ok(result)
    }

    fn get_edge_distance(
        &self,
        edge_id: EdgeId,
        distance_unit: Option<DistanceUnit>,
    ) -> Result<Distance, CompassAppError> {
        let op = move |g: &Graph| {
            let edge = g.get_edge(edge_id).map_err(CompassAppError::GraphError)?;
            Ok(edge.distance)
        };
        let result_base: Distance = graph_op(self, &op)?;
        let result = match distance_unit {
            Some(du) => DistanceUnit::Meters.convert(result_base, du),
            None => result_base,
        };
        Ok(result)
    }

    fn get_incident_edge_ids(
        &self,
        vertex_id: VertexId,
        direction: Direction,
    ) -> Result<Vec<EdgeId>, CompassAppError> {
        let op = move |g: &Graph| {
            let incident_edges = g
                .incident_edges(vertex_id, direction)
                .map_err(CompassAppError::GraphError)?;
            Ok(incident_edges)
        };
        let result: Vec<EdgeId> = graph_op(self, &op)?;
        Ok(result)
    }
}

fn graph_op<T>(
    app: &SearchApp,
    op: &dyn Fn(&Graph) -> Result<T, CompassAppError>,
) -> Result<T, CompassAppError>
where
    T: Send,
{
    let g_ref = app.get_graph_reference();
    let g = g_ref
        .read()
        .map_err(|e| CompassAppError::ReadOnlyPoisonError(e.to_string()))?;
    let result = op(&g)?;
    Ok(result)
}
