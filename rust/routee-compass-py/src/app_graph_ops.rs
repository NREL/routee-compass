use crate::app_wrapper::CompassAppWrapper;
use pyo3::{exceptions::PyException, PyResult};
use routee_compass::app::search::search_app_graph_ops::SearchAppGraphOps;
use routee_compass_core::{
    algorithm::search::direction::Direction,
    model::road_network::{edge_id::EdgeId, vertex_id::VertexId},
    model::unit::{as_f64::AsF64, DistanceUnit},
};
use std::str::FromStr;

pub fn graph_edge_origin(app: &CompassAppWrapper, edge_id: usize) -> PyResult<usize> {
    let edge_id_internal = EdgeId(edge_id);
    app.routee_compass
        .search_app
        .get_edge_origin(edge_id_internal)
        .map(|o| o.0)
        .map_err(|e| {
            PyException::new_err(format!(
                "error retrieving edge origin for edge_id {}: {}",
                edge_id, e
            ))
        })
}

pub fn graph_edge_destination(app: &CompassAppWrapper, edge_id: usize) -> PyResult<usize> {
    let edge_id_internal = EdgeId(edge_id);
    app.routee_compass
        .search_app
        .get_edge_destination(edge_id_internal)
        .map(|o| o.0)
        .map_err(|e| {
            PyException::new_err(format!(
                "error retrieving edge destination for edge_id {}: {}",
                edge_id, e
            ))
        })
}

pub fn graph_edge_distance(
    app: &CompassAppWrapper,
    edge_id: usize,
    distance_unit: Option<String>,
) -> PyResult<f64> {
    let du_internal_result: PyResult<Option<DistanceUnit>> = match distance_unit {
        Some(du_str) => {
            let du = DistanceUnit::from_str(du_str.as_str()).map_err(|_| {
                PyException::new_err(format!("could not deserialize distance unit '{}'", du_str))
            })?;

            Ok(Some(du))
        }

        None => Ok(None),
    };
    let du_internal = du_internal_result?;
    let edge_id_internal = EdgeId(edge_id);
    app.routee_compass
        .search_app
        .get_edge_distance(edge_id_internal, du_internal)
        .map(|o| o.as_f64())
        .map_err(|e| {
            PyException::new_err(format!(
                "error retrieving edge destination for edge_id {}: {}",
                edge_id, e
            ))
        })
}

pub fn get_out_edge_ids(app: &CompassAppWrapper, vertex_id: usize) -> PyResult<Vec<usize>> {
    let vertex_id_internal = VertexId(vertex_id);
    app.routee_compass
        .search_app
        .get_incident_edge_ids(vertex_id_internal, Direction::Forward)
        .map(|es| es.iter().map(|e| e.0).collect())
        .map_err(|e| {
            PyException::new_err(format!(
                "error retrieving out edges for vertex_id {}: {}",
                vertex_id, e
            ))
        })
}

pub fn get_in_edge_ids(app: &CompassAppWrapper, vertex_id: usize) -> PyResult<Vec<usize>> {
    let vertex_id_internal = VertexId(vertex_id);
    app.routee_compass
        .search_app
        .get_incident_edge_ids(vertex_id_internal, Direction::Reverse)
        .map(|es| es.iter().map(|e| e.0).collect())
        .map_err(|e| {
            PyException::new_err(format!(
                "error retrieving in edges for vertex_id {}: {}",
                vertex_id, e
            ))
        })
}
