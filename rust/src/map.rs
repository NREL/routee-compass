use rstar::RTree;

use anyhow::Result;
use pyo3::{prelude::*, types::PyType};
use rayon::prelude::*;

use crate::{
    algorithm::{build_restriction_function, dijkstra_shortest_path},
    graph::{Graph, Link, Node},
    powertrain::{
        build_routee_cost_function, build_routee_cost_function_with_tods, VehicleParameters,
        ROUTEE_SCALE_FACTOR,
    },
    time_of_day_speed::{DayOfWeek, SecondOfDay, TimeOfDaySpeeds},
};

#[pyclass]
#[derive(Clone)]
pub struct SearchInput {
    #[pyo3(get)]
    pub search_id: String,
    #[pyo3(get)]
    pub origin: [isize; 2],
    #[pyo3(get)]
    pub destination: [isize; 2],
    #[pyo3(get)]
    pub second_of_day: SecondOfDay,
    #[pyo3(get)]
    pub day_of_week: DayOfWeek,
    #[pyo3(get)]
    pub vehicle_parameters: Option<VehicleParameters>,
}

impl Default for SearchInput {
    fn default() -> Self {
        SearchInput {
            search_id: "default".to_string(),
            origin: [0, 0],
            destination: [0, 0],
            second_of_day: 0,
            day_of_week: 0,
            vehicle_parameters: None,
        }
    }
}

#[pymethods]
impl SearchInput {
    #[new]
    pub fn new(
        search_id: String,
        origin: [isize; 2],
        destination: [isize; 2],
        second_of_day: Option<SecondOfDay>,
        day_of_week: Option<DayOfWeek>,
        vehicle_parameters: Option<VehicleParameters>,
    ) -> Self {
        SearchInput {
            search_id,
            origin,
            destination,
            second_of_day: second_of_day.unwrap_or(Default::default()),
            day_of_week: day_of_week.unwrap_or(Default::default()),
            vehicle_parameters,
        }
    }
}

#[pyclass]
pub struct SearchResult {
    #[pyo3(get)]
    pub search_id: String,
    #[pyo3(get)]
    pub metric: f64,
    #[pyo3(get)]
    pub metric_name: String,
    #[pyo3(get)]
    pub path: Vec<Link>,
}

#[pyclass]
pub struct RustMap {
    #[pyo3(get)]
    pub graph: Graph,
    pub rtree: RTree<Node>,

    #[pyo3(get)]
    pub max_search_attempts: usize,
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
        let nodes = graph.nodes.values().cloned().collect::<Vec<Node>>();
        let rtree = RTree::bulk_load(nodes);
        RustMap {
            graph,
            rtree,
            max_search_attempts: 10,
        }
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

    pub fn get_closest_nodes(&self, point: [isize; 2], n: usize) -> Vec<Node> {
        self.rtree
            .nearest_neighbor_iter(&point)
            .take(n)
            .cloned()
            .collect()
    }

    /// shortest path from start to end using the time as the weight;
    /// returns the path as a list of links
    pub fn shortest_time_path(&self, search_input: SearchInput) -> Option<SearchResult> {
        let start_node = self.get_closest_node(search_input.origin)?;
        let end_node = self.get_closest_node(search_input.destination)?;
        match dijkstra_shortest_path(
            &self.graph,
            &start_node.id,
            &end_node.id,
            |link| link.time_seconds(),
            build_restriction_function(search_input.vehicle_parameters),
        ) {
            Some((time_seconds, path)) => Some(SearchResult {
                search_id: search_input.search_id,
                metric: time_seconds as f64,
                metric_name: "time_seconds".to_string(),
                path: self.graph.get_links_in_path(path),
            }),
            None => None,
        }
    }
    /// shortest path from start to end using the time (based on time of day) as the weight;
    /// returns the path as a list of links
    pub fn shortest_time_path_by_time_of_day(
        &self,
        search_input: SearchInput,
        time_of_day_speed: &TimeOfDaySpeeds,
    ) -> Option<SearchResult> {
        let start_node = self.get_closest_node(search_input.origin)?;
        let end_node = self.get_closest_node(search_input.destination)?;
        match dijkstra_shortest_path(
            &self.graph,
            &start_node.id,
            &end_node.id,
            |link| {
                time_of_day_speed.link_time_seconds_by_time_of_day(
                    link,
                    search_input.second_of_day,
                    search_input.day_of_week,
                )
            },
            build_restriction_function(search_input.vehicle_parameters),
        ) {
            Some((time_seconds, path)) => Some(SearchResult {
                search_id: search_input.search_id,
                metric: time_seconds as f64,
                metric_name: "time_seconds".to_string(),
                path: self.graph.get_links_in_path(path),
            }),
            None => None,
        }
    }

    /// shortest path the energy from a routee model as the weight;
    pub fn shortest_energy_path(
        &self,
        search_input: SearchInput,
        routee_model_path: &str,
    ) -> Option<SearchResult> {
        let start_node = self.get_closest_node(search_input.origin)?;
        let end_node = self.get_closest_node(search_input.destination)?;
        let routee_cost_function =
            build_routee_cost_function(routee_model_path, search_input.vehicle_parameters).unwrap();
        match dijkstra_shortest_path(
            &self.graph,
            &start_node.id,
            &end_node.id,
            routee_cost_function,
            build_restriction_function(search_input.vehicle_parameters),
        ) {
            Some((scaled_energy, path)) => {
                let energy = scaled_energy as f64 / ROUTEE_SCALE_FACTOR;
                Some(SearchResult {
                    search_id: search_input.search_id,
                    metric: energy,
                    metric_name: "energy_gallons_gas".to_string(),
                    path: self.graph.get_links_in_path(path),
                })
            }
            None => None,
        }
    }

    pub fn shortest_energy_path_by_time_of_day(
        &self,
        search_input: SearchInput,
        routee_model_path: &str,
        time_of_day_speed: TimeOfDaySpeeds,
    ) -> Option<SearchResult> {
        let start_node = self.get_closest_node(search_input.origin)?;
        let end_node = self.get_closest_node(search_input.destination)?;
        let routee_cost_function = build_routee_cost_function_with_tods(
            routee_model_path,
            time_of_day_speed,
            search_input.second_of_day,
            search_input.day_of_week,
            search_input.vehicle_parameters,
        )
        .unwrap();
        match dijkstra_shortest_path(
            &self.graph,
            &start_node.id,
            &end_node.id,
            routee_cost_function,
            build_restriction_function(search_input.vehicle_parameters),
        ) {
            Some((scaled_energy, path)) => {
                let energy = scaled_energy as f64 / ROUTEE_SCALE_FACTOR;
                Some(SearchResult {
                    search_id: search_input.search_id,
                    metric: energy,
                    metric_name: "energy_gallons_gas".to_string(),
                    path: self.graph.get_links_in_path(path),
                })
            }

            None => None,
        }
    }

    pub fn parallel_shortest_time_path(
        &self,
        search_inputs: Vec<SearchInput>,
    ) -> Vec<Option<SearchResult>> {
        search_inputs
            .into_par_iter()
            .map(|input| self.shortest_time_path(input))
            .collect()
    }

    pub fn parallel_shortest_time_path_by_time_of_day(
        &self,
        search_inputs: Vec<SearchInput>,
        time_of_day_speed: &TimeOfDaySpeeds,
    ) -> Vec<Option<SearchResult>> {
        search_inputs
            .into_par_iter()
            .map(|search_input| {
                self.shortest_time_path_by_time_of_day(search_input, time_of_day_speed)
            })
            .collect()
    }

    pub fn parallel_shortest_energy_path(
        &self,
        search_inputs: Vec<SearchInput>,
        routee_model_path: &str,
    ) -> Vec<Option<SearchResult>> {
        search_inputs
            .into_par_iter()
            .map(|search_input| self.shortest_energy_path(search_input, routee_model_path))
            .collect()
    }

    pub fn parallel_shortest_energy_path_by_time_of_day(
        &self,
        search_inputs: Vec<SearchInput>,
        routee_model_path: &str,
        time_of_day_speed: TimeOfDaySpeeds,
    ) -> Vec<Option<SearchResult>> {
        search_inputs
            .into_par_iter()
            .map(|search_input| {
                self.shortest_energy_path_by_time_of_day(
                    search_input,
                    routee_model_path,
                    time_of_day_speed.clone(),
                )
            })
            .collect()
    }
}
