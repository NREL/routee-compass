use super::search_app_result::SearchAppResult;
use crate::{app::app_error::AppError, plugin::input::input_json_extensions::InputJsonExtensions};
use async_std::{future::timeout, task};
use chrono::Local;
use compass_core::{
    algorithm::search::min_search_tree::{
        a_star::{
            a_star::{backtrack, backtrack_edges, run_a_star, run_a_star_edge_oriented},
            cost_estimate_function::CostEstimateFunction,
        },
        direction::Direction,
    },
    model::{
        frontier::frontier_model::FrontierModel,
        graph::{directed_graph::DirectedGraph, edge_id::EdgeId},
        traversal::traversal_model::TraversalModel,
    },
    util::read_only_lock::{DriverReadOnlyLock, ExecutorReadOnlyLock},
};
use rayon::prelude::*;
use std::time;
use std::{sync::Arc, time::Duration};

pub struct SearchApp {
    graph: Arc<DriverReadOnlyLock<Box<dyn DirectedGraph>>>,
    a_star_heuristic: Arc<DriverReadOnlyLock<Box<dyn CostEstimateFunction>>>,
    traversal_model: Arc<DriverReadOnlyLock<Box<dyn TraversalModel>>>,
    frontier_model: Arc<DriverReadOnlyLock<Box<dyn FrontierModel>>>,
    pub parallelism: usize,
    pub query_timeout_ms: u64,
}

impl SearchApp {
    /// builds a new CompassApp from the required components.
    /// handles all of the specialized boxing that allows for simple parallelization.
    pub fn new(
        graph: Box<dyn DirectedGraph>,
        traversal_model: Box<dyn TraversalModel>,
        frontier_model: Box<dyn FrontierModel>,
        a_star_heuristic: Box<dyn CostEstimateFunction>,
        parallelism: Option<usize>,
        query_timeout_ms: Option<u64>,
    ) -> Self {
        let g = Arc::new(DriverReadOnlyLock::new(graph));
        let h = Arc::new(DriverReadOnlyLock::new(a_star_heuristic));
        let t = Arc::new(DriverReadOnlyLock::new(traversal_model));
        let f = Arc::new(DriverReadOnlyLock::new(frontier_model));
        let parallelism_or_default = parallelism.unwrap_or(rayon::current_num_threads());
        let query_timeout_ms_or_default = query_timeout_ms.unwrap_or(2000);
        return SearchApp {
            graph: g,
            a_star_heuristic: h,
            traversal_model: t,
            frontier_model: f,
            parallelism: parallelism_or_default,
            query_timeout_ms: query_timeout_ms_or_default,
        };
    }

    ///
    /// runs a set of queries in parallel against the state of this CompassApp
    ///
    pub fn run_vertex_oriented(
        &self,
        queries: &Vec<serde_json::Value>,
    ) -> Result<Vec<Result<SearchAppResult, AppError>>, AppError> {
        // let _pool = rayon::ThreadPoolBuilder::new()
        //     .num_threads(self.parallelism)
        //     .build()
        //     .map_err(|e| {
        //         AppError::InternalError(format!("failure getting thread pool: {}", e.to_string()))
        //     })?;
        // execute the route search
        let result: Vec<Result<SearchAppResult, AppError>> = queries
            .clone()
            .into_iter()
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
                let cost_inner = Arc::new(self.a_star_heuristic.read_only());
                let result = task::block_on(timeout(Duration::from_millis(2000), async {
                    run_a_star(
                        Direction::Forward,
                        o,
                        d,
                        dg_inner,
                        tm_inner,
                        fm_inner,
                        cost_inner,
                    )
                }));
                match result {
                    Err(e) => Err(AppError::TimeoutError(e)),
                    Ok(tree_or_error) => {
                        let tree = tree_or_error.map_err(AppError::SearchError)?;
                        let search_end_time = Local::now();
                        let search_runtime = (search_end_time - search_start_time)
                            .to_std()
                            .unwrap_or(time::Duration::ZERO);
                        log::debug!(
                            "finished query for {} -> {} in {:?} milliseconds",
                            o,
                            d,
                            search_runtime.as_millis()
                        );
                        let route_start_time = Local::now();
                        let route_result =
                            task::block_on(timeout(Duration::from_millis(2000), async {
                                backtrack(o, d, &tree)
                            }));
                        let route = match route_result {
                            Err(e) => return Err(AppError::TimeoutError(e)),
                            Ok(r) => r.map_err(AppError::SearchError),
                        }?;
                        let route_end_time = Local::now();
                        let route_runtime = (route_end_time - route_start_time)
                            .to_std()
                            .unwrap_or(time::Duration::ZERO);
                        log::debug!(
                            "backtracked route for {} -> {} in {:?} milliseconds",
                            o,
                            d,
                            route_runtime.as_millis()
                        );
                        Ok(SearchAppResult {
                            route,
                            tree,
                            search_runtime,
                            route_runtime,
                            total_runtime: search_runtime + route_runtime,
                        })
                    }
                }
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
                let cost_inner = Arc::new(self.a_star_heuristic.read_only());
                let result = task::block_on(timeout(
                    Duration::from_millis(self.query_timeout_ms),
                    async {
                        run_a_star_edge_oriented(
                            Direction::Forward,
                            o,
                            d,
                            dg_inner_search,
                            tm_inner,
                            fm_inner,
                            cost_inner,
                        )
                    },
                ));
                match result {
                    Err(e) => Err(AppError::TimeoutError(e)),
                    Ok(tree_or_error) => {
                        let tree = tree_or_error.map_err(AppError::SearchError)?;
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
                        Ok(SearchAppResult {
                            route,
                            tree,
                            search_runtime,
                            route_runtime,
                            total_runtime: search_runtime + route_runtime,
                        })
                    }
                }
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

    /// helper function for accessing the CostEstimateFunction
    ///
    /// example:
    ///
    /// let search_app: SearchApp = ...;
    /// let reference = search_app.get_a_star_heuristic_reference();
    /// let est_fn = reference.read();
    /// // do things with CostEstimateFunction
    pub fn get_a_star_heuristic_reference(
        &self,
    ) -> Arc<ExecutorReadOnlyLock<Box<dyn CostEstimateFunction>>> {
        Arc::new(self.a_star_heuristic.read_only())
    }
}
