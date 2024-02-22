use std::{path::Path, str::FromStr};

use routee_compass_core::{
    algorithm::search::direction::Direction,
    model::{
        road_network::{edge_id::EdgeId, vertex_id::VertexId},
        unit::{as_f64::AsF64, DistanceUnit},
    },
};

use super::compass::{compass_app::CompassApp, compass_app_error::CompassAppError};

use crate::app::search::search_app_graph_ops::SearchAppGraphOps;

pub trait CompassAppBindings {
    // Functions to be implemented
    fn new(app: CompassApp) -> Self
    where
        Self: Sized;
    fn app(&self) -> &CompassApp;

    // Default functions
    fn graph_edge_origin(&self, edge_id: usize) -> Result<usize, CompassAppError> {
        let edge_id_internal = EdgeId(edge_id);
        self.app()
            .search_app
            .get_edge_origin(edge_id_internal)
            .map(|o| o.0)
    }
    fn graph_edge_destination(&self, edge_id: usize) -> Result<usize, CompassAppError> {
        let edge_id_internal = EdgeId(edge_id);
        self.app()
            .search_app
            .get_edge_destination(edge_id_internal)
            .map(|o| o.0)
    }
    fn graph_edge_distance(
        &self,
        edge_id: usize,
        distance_unit: Option<String>,
    ) -> Result<f64, CompassAppError> {
        let du_internal: Option<DistanceUnit> = match distance_unit {
            Some(du_str) => {
                let du = DistanceUnit::from_str(du_str.as_str()).map_err(|_| {
                    CompassAppError::InternalError(format!(
                        "could not deserialize distance unit '{}'",
                        du_str
                    ))
                })?;

                Some(du)
            }

            None => None,
        };
        let edge_id_internal = EdgeId(edge_id);
        self.app()
            .search_app
            .get_edge_distance(edge_id_internal, du_internal)
            .map(|o| o.as_f64())
    }
    fn graph_get_out_edge_ids(&self, vertex_id: usize) -> Result<Vec<usize>, CompassAppError> {
        let vertex_id_internal = VertexId(vertex_id);
        self.app()
            .search_app
            .get_incident_edge_ids(vertex_id_internal, Direction::Forward)
            .map(|es| es.iter().map(|e| e.0).collect())
    }
    fn graph_get_in_edge_ids(&self, vertex_id: usize) -> Result<Vec<usize>, CompassAppError> {
        let vertex_id_internal = VertexId(vertex_id);
        self.app()
            .search_app
            .get_incident_edge_ids(vertex_id_internal, Direction::Reverse)
            .map(|es| es.iter().map(|e| e.0).collect())
    }
}
