use rstar::RTree;

use anyhow::Result;
use pyo3::{prelude::*, types::PyType};
use rayon::prelude::*;

use crate::{
    algorithm::{build_restriction_function, dijkstra_shortest_path},
    graph::{Graph, Link, Node},
    powertrain::{
        build_routee_cost_function_with_tods, compute_energy_over_path, VehicleParameters,
        ROUTEE_SCALE_FACTOR,
    },
    time_of_day_speed::{DayOfWeek, SecondOfDay, TimeOfDaySpeeds},
};

#[pyclass]
#[derive(Clone)]
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
    pub search_type: SearchType,
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
}

impl Default for SearchInput {
    fn default() -> Self {
        SearchInput {
            search_id: "default".to_string(),
            search_type: SearchType::ShortestTime,
            origin: [0, 0],
            destination: [0, 0],
            second_of_day: 0,
            day_of_week: 0,
            time_of_day_speeds: Default::default(),
            vehicle_parameters: None,
            routee_model_path: None,
        }
    }
}

#[pymethods]
impl SearchInput {
    #[new]
    pub fn new(
        search_id: String,
        search_type: SearchType,
        origin: [isize; 2],
        destination: [isize; 2],
        second_of_day: Option<SecondOfDay>,
        day_of_week: Option<DayOfWeek>,
        time_of_day_speeds: Option<TimeOfDaySpeeds>,
        vehicle_parameters: Option<VehicleParameters>,
        routee_model_path: Option<String>,
    ) -> Self {
        SearchInput {
            search_id,
            search_type,
            origin,
            destination,
            second_of_day: second_of_day.unwrap_or_default(),
            day_of_week: day_of_week.unwrap_or_default(),
            time_of_day_speeds: time_of_day_speeds.unwrap_or_default(),
            vehicle_parameters,
            routee_model_path,
        }
    }
}

#[pyclass]
pub struct SearchResult {
    #[pyo3(get)]
    pub search_id: String,
    #[pyo3(get)]
    pub time_seconds: u32,
    #[pyo3(get)]
    pub energy_gallons_gas: f64,
    #[pyo3(get)]
    pub path: Vec<Link>,
}

pub fn compute_time_seconds_over_path(path: &Vec<Link>, search_input: &SearchInput) -> u32 {
    path.iter()
        .map(|link| {
            search_input
                .time_of_day_speeds
                .link_time_seconds_by_time_of_day(
                    link,
                    search_input.second_of_day,
                    search_input.day_of_week,
                )
        })
        .sum()
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

    pub fn shortest_path(&self, search_input: SearchInput) -> Option<SearchResult> {
        match search_input.search_type {
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
            |link| {
                search_input
                    .time_of_day_speeds
                    .link_time_seconds_by_time_of_day(
                        link,
                        search_input.second_of_day,
                        search_input.day_of_week,
                    )
            },
            build_restriction_function(search_input.vehicle_parameters),
        ) {
            Some((time_seconds, node_path)) => {
                let path = self.graph.get_links_in_path(node_path);
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
        let routee_cost_function = build_routee_cost_function_with_tods(search_input.clone()).unwrap();
        match dijkstra_shortest_path(
            &self.graph,
            &start_node.id,
            &end_node.id,
            routee_cost_function,
            build_restriction_function(search_input.vehicle_parameters),
        ) {
            Some((scaled_energy, nodes_in_path)) => {
                let energy = scaled_energy as f64 / ROUTEE_SCALE_FACTOR;
                let path = self.graph.get_links_in_path(nodes_in_path);
                let time_seconds = compute_time_seconds_over_path(&path, &search_input);
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
    ) -> Vec<Option<SearchResult>> {
        search_inputs
            .into_par_iter()
            .map(|input| self.shortest_path(input))
            .collect()
    }
}
