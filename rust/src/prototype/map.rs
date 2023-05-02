use rstar::RTree;

use anyhow::Result;
use pyo3::{prelude::*, types::PyType};
use rayon::prelude::*;

use crate::prototype::{
    algorithm::{build_restriction_function, dijkstra_shortest_path},
    graph::{Graph, Link, Node},
    powertrain::{build_routee_cost_function, VehicleParameters, ROUTEE_SCALE_FACTOR},
};

#[pyclass]
pub struct RustMap {
    #[pyo3(get)]
    pub graph: Graph,
    pub rtree: RTree<Node>,
}

impl RustMap {
    pub fn from_file(path: &str) -> Result<Self> {
        let graph = Graph::from_file(path)?;
        Ok(RustMap::new(graph))
    }
}

#[pymethods]
impl RustMap {
    #[new]
    pub fn new(graph: Graph) -> Self {
        let rtree = RTree::bulk_load(graph.get_nodes().clone());
        RustMap { graph, rtree }
    }

    pub fn to_file(&self, path: &str) -> Result<()> {
        self.graph.to_file(path)
    }

    #[classmethod]
    #[pyo3(name = "from_file")]
    pub fn py_from_file(_: &PyType, path: &str) -> Result<Self> {
        let graph = Graph::from_file(path)?;
        Ok(RustMap::new(graph))
    }

    pub fn get_closest_node(&self, point: [isize; 2]) -> Option<Node> {
        self.rtree.nearest_neighbor(&point).cloned()
    }

    /// shortest path from start to end using the time as the weight;
    /// returns the path as a list of links
    pub fn shortest_time_path(
        &self,
        start: [isize; 2],
        end: [isize; 2],
        vehicle_parameters: Option<VehicleParameters>,
    ) -> Option<(u32, Vec<Link>)> {
        let start_node = self.get_closest_node(start)?;
        let end_node = self.get_closest_node(end)?;
        match dijkstra_shortest_path(
            &self.graph,
            &start_node,
            &end_node,
            |link| link.time_seconds,
            build_restriction_function(vehicle_parameters),
        ) {
            Some((time_seconds, path)) => Some((time_seconds, self.graph.get_links_in_path(path))),
            None => None,
        }
    }

    /// shortest path the energy from a routee model as the weight;
    pub fn shortest_energy_path(
        &self,
        start: [isize; 2],
        end: [isize; 2],
        routee_model_path: &str,
        vehicle_parameters: Option<VehicleParameters>,
    ) -> Option<(f64, Vec<Link>)> {
        let start_node = self.get_closest_node(start)?;
        let end_node = self.get_closest_node(end)?;
        let routee_cost_function = build_routee_cost_function(routee_model_path).unwrap();
        match dijkstra_shortest_path(
            &self.graph,
            &start_node,
            &end_node,
            routee_cost_function,
            build_restriction_function(vehicle_parameters),
        ) {
            Some((scaled_energy, path)) => {
                let energy = scaled_energy as f64 / ROUTEE_SCALE_FACTOR;
                Some((energy, self.graph.get_links_in_path(path)))
            }

            None => None,
        }
    }

    pub fn parallel_shortest_time_path(
        &self,
        od_pairs: Vec<([isize; 2], [isize; 2])>,
        vehicle_parameters: Option<VehicleParameters>,
    ) -> Vec<Option<(u32, Vec<Link>)>> {
        od_pairs
            .into_par_iter()
            .map(|(start, end)| self.shortest_time_path(start, end, vehicle_parameters))
            .collect()
    }

    pub fn parallel_shortest_energy_path(
        &self,
        od_pairs: Vec<([isize; 2], [isize; 2])>,
        routee_model_path: &str,
        vehicle_parameters: Option<VehicleParameters>,
    ) -> Vec<Option<(f64, Vec<Link>)>> {
        od_pairs
            .into_par_iter()
            .map(|(start, end)| {
                self.shortest_energy_path(start, end, routee_model_path, vehicle_parameters)
            })
            .collect()
    }
}
