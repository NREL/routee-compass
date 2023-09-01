use super::search_app_result::SearchAppResult;
use crate::{app::app_error::AppError, plugin::input::input_json_extensions::InputJsonExtensions};
use chrono::Local;
use compass_core::{
    algorithm::search::min_search_tree::{
        a_star::a_star::{backtrack, backtrack_edges, run_a_star, run_a_star_edge_oriented},
        direction::Direction,
    },
    model::{
        frontier::frontier_model::FrontierModel,
        graph::{directed_graph::DirectedGraph, edge_id::EdgeId},
        termination::termination_model::TerminationModel,
        traversal::traversal_model::TraversalModel,
    },
    util::read_only_lock::{DriverReadOnlyLock, ExecutorReadOnlyLock},
};
use rayon::prelude::*;
use std::time;
use std::{sync::Arc, time::Duration};

pub struct SearchApp {
    graph: Arc<DriverReadOnlyLock<Box<dyn DirectedGraph>>>,
    traversal_model: Arc<DriverReadOnlyLock<Box<dyn TraversalModel>>>,
    frontier_model: Arc<DriverReadOnlyLock<Box<dyn FrontierModel>>>,
    termination_model: Arc<DriverReadOnlyLock<Box<dyn TerminationModel>>>,
    pub parallelism: usize,
    pub query_timeout_ms: u64,
    pub include_tree: bool,
}

impl SearchApp {
    /// builds a new CompassApp from the required components.
    /// handles all of the specialized boxing that allows for simple parallelization.
    pub fn new(
        graph: Box<dyn DirectedGraph>,
        traversal_model: Box<dyn TraversalModel>,
        frontier_model: Box<dyn FrontierModel>,
        termination_model: Box<dyn TerminationModel>,
        parallelism: Option<usize>,
        query_timeout_ms: Option<u64>,
        include_tree: bool,
    ) -> Self {
        let g = Arc::new(DriverReadOnlyLock::new(graph));
        let t = Arc::new(DriverReadOnlyLock::new(traversal_model));
        let f = Arc::new(DriverReadOnlyLock::new(frontier_model));
        let r = Arc::new(DriverReadOnlyLock::new(termination_model));
        let parallelism_or_default = parallelism.unwrap_or(rayon::current_num_threads());
        let query_timeout_ms_or_default = query_timeout_ms.unwrap_or(2000);
        return SearchApp {
            graph: g,
            traversal_model: t,
            frontier_model: f,
            termination_model: r,
            parallelism: parallelism_or_default,
            query_timeout_ms: query_timeout_ms_or_default,
            include_tree,
        };
    }

    ///
    /// runs a set of queries in parallel against the state of this CompassApp
    ///
    pub fn run_vertex_oriented(
        &self,
        queries: &Vec<serde_json::Value>,
    ) -> Result<Vec<Result<SearchAppResult, AppError>>, AppError> {
        log::debug!("Building thread pool with {} threads", self.parallelism);
        let _pool = rayon::ThreadPoolBuilder::new()
            .num_threads(self.parallelism)
            .build()
            .map_err(|e| {
                AppError::InternalError(format!("failure getting thread pool: {}", e.to_string()))
            })?;
        // execute the route search
        let result: Vec<Result<SearchAppResult, AppError>> = queries
            .clone()
            .into_par_iter()
            .map(|query| {
                log::debug!("Query: {}", query);
                let o = query.get_origin_vertex().map_err(AppError::PluginError)?;
                let d = query
                    .get_destination_vertex()
                    .map_err(AppError::PluginError)?;
                let search_start_time = Local::now();
                let dg_inner = Arc::new(self.graph.read_only());
                let tm_inner = Arc::new(self.traversal_model.read_only());
                let fm_inner = Arc::new(self.frontier_model.read_only());
                run_a_star(
                    Direction::Forward,
                    o,
                    d,
                    dg_inner,
                    tm_inner,
                    fm_inner,
                    Duration::from_millis(self.query_timeout_ms),
                )
                .and_then(|tree| {
                    let search_end_time = Local::now();
                    let search_runtime = (search_end_time - search_start_time)
                        .to_std()
                        .unwrap_or(time::Duration::ZERO);
                    log::debug!(
                        "Search Completed in {:?} miliseconds",
                        search_runtime.as_millis()
                    );
                    let route_start_time = Local::now();
                    let route = backtrack(o, d, &tree)?;
                    let route_end_time = Local::now();
                    let route_runtime = (route_end_time - route_start_time)
                        .to_std()
                        .unwrap_or(time::Duration::ZERO);
                    log::debug!(
                        "Route Computed in {:?} miliseconds",
                        route_runtime.as_millis()
                    );
                    match self.include_tree {
                        true => Ok(SearchAppResult {
                            route,
                            tree: Some(tree),
                            search_runtime,
                            route_runtime,
                            total_runtime: search_runtime + route_runtime,
                        }),
                        false => Ok(SearchAppResult {
                            route,
                            tree: None,
                            search_runtime,
                            route_runtime,
                            total_runtime: search_runtime + route_runtime,
                        }),
                    }
                })
                .map_err(AppError::SearchError)
            })
            .collect();

        return Ok(result);
    }

    ///
    /// runs a set of queries in parallel against the state of this CompassApp
    ///
    pub fn run_edge_oriented(
        &self,
        queries: Vec<(EdgeId, EdgeId)>,
    ) -> Result<Vec<Result<SearchAppResult, AppError>>, AppError> {
        let _pool = rayon::ThreadPoolBuilder::new()
            .num_threads(self.parallelism)
            .build()
            .map_err(|e| {
                AppError::InternalError(format!("failure getting thread pool: {}", e.to_string()))
            })?;
        // execute the route search
        let result: Vec<Result<SearchAppResult, AppError>> = queries
            .clone()
            .into_par_iter()
            .map(|(o, d)| {
                let search_start_time = Local::now();
                let dg_inner_search = Arc::new(self.graph.read_only());
                let dg_inner_backtrack = Arc::new(self.graph.read_only());
                let tm_inner = Arc::new(self.traversal_model.read_only());
                let fm_inner = Arc::new(self.frontier_model.read_only());
                run_a_star_edge_oriented(
                    Direction::Forward,
                    o,
                    d,
                    dg_inner_search,
                    tm_inner,
                    fm_inner,
                    Duration::from_millis(self.query_timeout_ms),
                )
                .and_then(|tree| {
                    let search_end_time = Local::now();
                    let route_start_time = Local::now();
                    let route = backtrack_edges(o, d, &tree, dg_inner_backtrack)?;
                    let route_end_time = Local::now();
                    let search_runtime = (search_end_time - search_start_time)
                        .to_std()
                        .unwrap_or(time::Duration::ZERO);
                    let route_runtime = (route_end_time - route_start_time)
                        .to_std()
                        .unwrap_or(time::Duration::ZERO);
                    match self.include_tree {
                        true => Ok(SearchAppResult {
                            route,
                            tree: Some(tree),
                            search_runtime,
                            route_runtime,
                            total_runtime: search_runtime + route_runtime,
                        }),
                        false => Ok(SearchAppResult {
                            route,
                            tree: None,
                            search_runtime,
                            route_runtime,
                            total_runtime: search_runtime + route_runtime,
                        }),
                    }
                })
                .map_err(AppError::SearchError)
            })
            .collect();

        return Ok(result);
    }

    /// helper function for accessing the DirectedGraph
    ///
    /// example:
    ///
    /// let search_app: SearchApp = ...;
    /// let reference = search_app.get_directed_graph_reference();
    /// let graph = reference.read();
    /// // do things with graph
    pub fn get_directed_graph_reference(
        &self,
    ) -> Arc<ExecutorReadOnlyLock<Box<dyn DirectedGraph>>> {
        Arc::new(self.graph.read_only())
    }

    /// helper function for accessing the TraversalModel
    ///
    /// example:
    ///
    /// let search_app: SearchApp = ...;
    /// let reference = search_app.get_traversal_model_reference();
    /// let traversal_model = reference.read();
    /// // do things with TraversalModel
    pub fn get_traversal_model_reference(
        &self,
    ) -> Arc<ExecutorReadOnlyLock<Box<dyn TraversalModel>>> {
        Arc::new(self.traversal_model.read_only())
    }
}
