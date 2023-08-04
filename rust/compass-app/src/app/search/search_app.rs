use super::search_app_result::SearchAppResult;
use crate::app::app_error::AppError;
use compass_core::{
    algorithm::search::min_search_tree::{
        a_star::{
            a_star::{backtrack, backtrack_edges, run_a_star, run_a_star_edge_oriented},
            cost_estimate_function::CostEstimateFunction,
        },
        direction::Direction,
    },
    model::{
        graph::{directed_graph::DirectedGraph, edge_id::EdgeId, vertex_id::VertexId},
        traversal::traversal_model::TraversalModel,
    },
    util::read_only_lock::{DriverReadOnlyLock, ExecutorReadOnlyLock},
};
use rayon::prelude::*;
use std::sync::Arc;

pub struct SearchApp<'app> {
    graph: Arc<DriverReadOnlyLock<&'app dyn DirectedGraph>>,
    a_star_heuristic: Arc<DriverReadOnlyLock<&'app dyn CostEstimateFunction>>,
    traversal_model: Arc<DriverReadOnlyLock<&'app TraversalModel>>,
    processes: usize,
}

impl<'app> SearchApp<'app> {
    /// builds a new CompassApp from the required components.
    /// handles all of the specialized boxing that allows for simple parallelization.
    pub fn new(
        graph: &'app dyn DirectedGraph,
        traversal_model: &'app TraversalModel,
        a_star_heuristic: &'app dyn CostEstimateFunction,
        processes: Option<usize>,
    ) -> Self {
        let g = Arc::new(DriverReadOnlyLock::new(graph as &dyn DirectedGraph));
        let h = Arc::new(DriverReadOnlyLock::new(
            a_star_heuristic as &dyn CostEstimateFunction,
        ));
        let t = Arc::new(DriverReadOnlyLock::new(traversal_model));
        let procs = processes.unwrap_or(rayon::current_num_threads());
        return SearchApp {
            graph: g,
            a_star_heuristic: h,
            traversal_model: t,
            processes: procs,
        };
    }

    ///
    /// runs a set of queries in parallel against the state of this CompassApp
    ///
    pub fn run_vertex_oriented(
        &self,
        queries: Vec<(VertexId, VertexId)>,
    ) -> Result<Vec<Result<SearchAppResult<VertexId>, AppError>>, AppError> {
        let _pool = rayon::ThreadPoolBuilder::new()
            .num_threads(self.processes)
            .build()
            .map_err(|e| {
                AppError::InternalError(format!("failure getting thread pool: {}", e.to_string()))
            })?;
        // execute the route search
        let result: Vec<Result<SearchAppResult<VertexId>, AppError>> = queries
            .clone()
            .into_par_iter()
            .map(|(o, d)| {
                let dg_inner = Arc::new(self.graph.read_only());
                let tm_inner = Arc::new(self.traversal_model.read_only());
                let cost_inner = Arc::new(self.a_star_heuristic.read_only());
                run_a_star(Direction::Forward, o, d, dg_inner, tm_inner, cost_inner)
                    .and_then(|tree| {
                        let tree_size = tree.len();
                        let route = backtrack(o, d, tree)?;
                        Ok(SearchAppResult {
                            origin: o,
                            destination: d,
                            route,
                            tree_size,
                        })
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
    ) -> Result<Vec<Result<SearchAppResult<EdgeId>, AppError>>, AppError> {
        let _pool = rayon::ThreadPoolBuilder::new()
            .num_threads(self.processes)
            .build()
            .map_err(|e| {
                AppError::InternalError(format!("failure getting thread pool: {}", e.to_string()))
            })?;
        // execute the route search
        let result: Vec<Result<SearchAppResult<EdgeId>, AppError>> = queries
            .clone()
            .into_par_iter()
            .map(|(o, d)| {
                let dg_inner_search = Arc::new(self.graph.read_only());
                let dg_inner_backtrack = Arc::new(self.graph.read_only());
                let tm_inner = Arc::new(self.traversal_model.read_only());
                let cost_inner = Arc::new(self.a_star_heuristic.read_only());
                run_a_star_edge_oriented(
                    Direction::Forward,
                    o,
                    d,
                    dg_inner_search,
                    tm_inner,
                    cost_inner,
                )
                .and_then(|tree| {
                    let tree_size = tree.len();
                    let route = backtrack_edges(o, d, tree, dg_inner_backtrack)?;
                    Ok(SearchAppResult {
                        origin: o,
                        destination: d,
                        route,
                        tree_size,
                    })
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
    ) -> Arc<ExecutorReadOnlyLock<&'app dyn DirectedGraph>> {
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
    pub fn get_traversal_model_reference(&self) -> Arc<ExecutorReadOnlyLock<&'app TraversalModel>> {
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
    ) -> Arc<ExecutorReadOnlyLock<&'app dyn CostEstimateFunction>> {
        Arc::new(self.a_star_heuristic.read_only())
    }
}
