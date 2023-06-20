use rstar::RTree;

use anyhow::Result;
use pyo3::{prelude::*, types::PyType};
use rayon::prelude::*;

use crate::prototype::{
    algorithm::{build_restriction_function, dijkstra_shortest_path},
    graph::{Graph, Link, Node},
    powertrain::{
        build_routee_cost_function_with_tods, compute_energy_over_path, VehicleParameters,
    },
    time_of_day_speed::{DayOfWeek, SecondOfDay, TimeOfDaySpeeds},
};

use super::algorithm::build_shortest_time_function;

#[pyclass]
#[derive(Clone, Copy)]
pub enum SearchType {
    ShortestTime,
    ShortestEnergy,
}

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
    pub time_of_day_speeds: TimeOfDaySpeeds,
    #[pyo3(get)]
    pub vehicle_parameters: Option<VehicleParameters>,
    #[pyo3(get)]
    pub routee_model_path: Option<String>,
    #[pyo3(get)]
    pub stop_cost_gallons_diesel: f64,
    #[pyo3(get)]
    pub stop_cost_time_seconds: u32,
    #[pyo3(get)]
    pub traffic_light_cost_time_seconds: u32,
}

impl Default for SearchInput {
    fn default() -> Self {
        SearchInput {
            search_id: "default".to_string(),
            origin: [0, 0],
            destination: [0, 0],
            second_of_day: 0,
            day_of_week: 0,
            time_of_day_speeds: Default::default(),
            vehicle_parameters: None,
            routee_model_path: None,
            stop_cost_gallons_diesel: 0.001,
            stop_cost_time_seconds: 10,
            traffic_light_cost_time_seconds: 5,
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
        time_of_day_speeds: Option<TimeOfDaySpeeds>,
        vehicle_parameters: Option<VehicleParameters>,
        routee_model_path: Option<String>,
        stop_cost_gallons_diesel: Option<f64>,
        stop_cost_time_seconds: Option<u32>,
        traffic_light_cost_time_seconds: Option<u32>,
    ) -> Self {
        SearchInput {
            search_id,
            origin,
            destination,
            second_of_day: second_of_day.unwrap_or_default(),
            day_of_week: day_of_week.unwrap_or_default(),
            time_of_day_speeds: time_of_day_speeds.unwrap_or_default(),
            vehicle_parameters,
            routee_model_path,
            stop_cost_gallons_diesel: stop_cost_gallons_diesel.unwrap_or_default(),
            stop_cost_time_seconds: stop_cost_time_seconds.unwrap_or_default(),
            traffic_light_cost_time_seconds: traffic_light_cost_time_seconds.unwrap_or_default(),
        }
    }
}

#[pyclass]
pub struct SearchResult {
    #[pyo3(get)]
    pub search_id: String,
    #[pyo3(get)]
    pub time_seconds: f64,
    #[pyo3(get)]
    pub energy_gallons_gas: f64,
    #[pyo3(get)]
    pub path: Vec<Link>,
}

pub fn compute_time_seconds_over_path(path: &Vec<Link>, search_input: &SearchInput) -> f64 {
    let mut time_seconds = 0.0;
    for link in path.iter() {
        let mut speed_kph = link.speed_kph as f64;
        if let Some(profile_id) = link.week_profile_ids[search_input.day_of_week] {
            let speed_modifier = search_input
                .time_of_day_speeds
                .get_modifier_by_second_of_day(profile_id, search_input.second_of_day);
            speed_kph *= speed_modifier;
        } 
        let distance_km = link.distance_centimeters as f64 / 100_000.0;
        let time_hours = distance_km / speed_kph;
        let time_seconds_link = time_hours * 3600.0;
        time_seconds += time_seconds_link;
    }
    time_seconds
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

    pub fn shortest_path(
        &self,
        search_input: SearchInput,
        search_type: SearchType,
    ) -> Option<SearchResult> {
        match search_type {
            SearchType::ShortestTime => self.shortest_time_path(search_input),
            SearchType::ShortestEnergy => self.shortest_energy_path(search_input),
        }
    }

    // shortest path from start to end using the time (based on time of day) as the weight;
    /// returns the path as a list of links
    pub fn shortest_time_path(&self, search_input: SearchInput) -> Option<SearchResult> {
        let start_node = self.get_closest_node(search_input.origin)?;
        let end_node = self.get_closest_node(search_input.destination)?;
        match dijkstra_shortest_path(
            &self.graph,
            &start_node.id,
            &end_node.id,
            build_shortest_time_function(search_input.clone()),
            build_restriction_function(search_input.vehicle_parameters),
        ) {
            Some((_, node_path)) => {
                let path = self.graph.get_links_in_path(node_path);
                let time_seconds = compute_time_seconds_over_path(&path, &search_input);
                let energy = compute_energy_over_path(&path, &search_input).unwrap();
                Some(SearchResult {
                    search_id: search_input.search_id,
                    time_seconds,
                    energy_gallons_gas: energy,
                    path,
                })
            }
            None => None,
        }
    }

    pub fn shortest_energy_path(&self, search_input: SearchInput) -> Option<SearchResult> {
        let start_node = self.get_closest_node(search_input.origin)?;
        let end_node = self.get_closest_node(search_input.destination)?;
        let routee_cost_function =
            build_routee_cost_function_with_tods(search_input.clone()).unwrap();
        match dijkstra_shortest_path(
            &self.graph,
            &start_node.id,
            &end_node.id,
            routee_cost_function,
            build_restriction_function(search_input.vehicle_parameters),
        ) {
            Some((scaled_energy, nodes_in_path)) => {
                let path = self.graph.get_links_in_path(nodes_in_path);
                let time_seconds = compute_time_seconds_over_path(&path, &search_input);
                let energy = compute_energy_over_path(&path, &search_input).unwrap();
                Some(SearchResult {
                    search_id: search_input.search_id,
                    time_seconds,
                    energy_gallons_gas: energy,
                    path,
                })
            }

            None => None,
        }
    }

    pub fn parallel_shortest_path(
        &self,
        search_inputs: Vec<SearchInput>,
        search_type: SearchType,
    ) -> Vec<Option<SearchResult>> {
        search_inputs
            .into_par_iter()
            .map(|input| self.shortest_path(input, search_type))
            .collect()
    }
}
