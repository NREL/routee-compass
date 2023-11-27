use super::search_app_result::SearchAppResult;
use crate::{
    app::compass::compass_app_error::CompassAppError,
    plugin::input::input_json_extensions::InputJsonExtensions,
};
use chrono::Local;
use routee_compass_core::{
    algorithm::search::{backtrack, search_algorithm::SearchAlgorithm},
    model::{
        frontier::frontier_model::FrontierModel,
        road_network::graph::Graph,
        termination::termination_model::TerminationModel,
        traversal::{
            traversal_model::TraversalModel, traversal_model_service::TraversalModelService,
        },
    },
    util::read_only_lock::{DriverReadOnlyLock, ExecutorReadOnlyLock},
};
use std::sync::Arc;
use std::time;

pub struct SearchApp {
    search_algorithm: SearchAlgorithm,
    graph: Arc<DriverReadOnlyLock<Graph>>,
    traversal_model_service: Arc<DriverReadOnlyLock<Arc<dyn TraversalModelService>>>,
    frontier_model: Arc<DriverReadOnlyLock<Box<dyn FrontierModel>>>,
    termination_model: Arc<DriverReadOnlyLock<TerminationModel>>,
}

impl SearchApp {
    /// builds a new SearchApp from the required components.
    /// handles all of the specialized boxing that allows for simple parallelization.
    pub fn new(
        search_algorithm: SearchAlgorithm,
        graph: Graph,
        traversal_model_service: Arc<dyn TraversalModelService>,
        frontier_model: Box<dyn FrontierModel>,
        termination_model: TerminationModel,
    ) -> Self {
        let graph = Arc::new(DriverReadOnlyLock::new(graph));
        let traversal_model_service = Arc::new(DriverReadOnlyLock::new(traversal_model_service));
        let frontier_model = Arc::new(DriverReadOnlyLock::new(frontier_model));
        let termination_model = Arc::new(DriverReadOnlyLock::new(termination_model));
        SearchApp {
            search_algorithm,
            graph,
            traversal_model_service,
            frontier_model,
            termination_model,
        }
    }

    /// runs a single vertex oriented query
    ///
    pub fn run_vertex_oriented(
        &self,
        query: &serde_json::Value,
    ) -> Result<SearchAppResult, CompassAppError> {
        let o = query
            .get_origin_vertex()
            .map_err(CompassAppError::PluginError)?;
        let d = query
            .get_destination_vertex()
            .map_err(CompassAppError::PluginError)?;
        let search_start_time = Local::now();
        let dg_inner = Arc::new(self.graph.read_only());

        let tm_inner = self
            .traversal_model_service
            .read_only()
            .read()
            .map_err(|e| CompassAppError::ReadOnlyPoisonError(e.to_string()))?
            .build(query)?;
        let fm_inner = Arc::new(self.frontier_model.read_only());
        let rm_inner = Arc::new(self.termination_model.read_only());
        self.search_algorithm
            .run_vertex_oriented(o, d, dg_inner, tm_inner, fm_inner, rm_inner)
            // run_a_star(o, Some(d), dg_inner, tm_inner, fm_inner, rm_inner)
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
                let route = match d {
                    None => vec![],
                    Some(dest) => backtrack::vertex_oriented_route(o, dest, &tree)?,
                };
                let route_end_time = Local::now();
                let route_runtime = (route_end_time - route_start_time)
                    .to_std()
                    .unwrap_or(time::Duration::ZERO);
                log::debug!(
                    "Route Computed in {:?} miliseconds",
                    route_runtime.as_millis()
                );
                Ok(SearchAppResult {
                    route,
                    tree,
                    search_start_time,
                    search_runtime,
                    route_runtime,
                    total_runtime: search_runtime + route_runtime,
                })
            })
            .map_err(CompassAppError::SearchError)
    }

    ///
    /// runs a single edge oriented query
    ///
    pub fn run_edge_oriented(
        &self,
        query: &serde_json::Value,
    ) -> Result<SearchAppResult, CompassAppError> {
        let o = query
            .get_origin_edge()
            .map_err(CompassAppError::PluginError)?;
        let d = query
            .get_destination_edge()
            .map_err(CompassAppError::PluginError)?;
        let search_start_time = Local::now();
        let dg_inner_search = Arc::new(self.graph.read_only());
        let dg_inner_backtrack = Arc::new(self.graph.read_only());
        let tm_inner = self
            .traversal_model_service
            .read_only()
            .read()
            .map_err(|e| CompassAppError::ReadOnlyPoisonError(e.to_string()))?
            .build(query)?;
        let fm_inner = Arc::new(self.frontier_model.read_only());
        let rm_inner = Arc::new(self.termination_model.read_only());
        self.search_algorithm
            .run_edge_oriented(o, d, dg_inner_search, tm_inner, fm_inner, rm_inner)
            .and_then(|tree| {
                let search_end_time = Local::now();
                let route_start_time = Local::now();
                let route = match d {
                    None => vec![],
                    Some(dest) => {
                        backtrack::edge_oriented_route(o, dest, &tree, dg_inner_backtrack)?
                    }
                };
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
                    search_start_time,
                    search_runtime,
                    route_runtime,
                    total_runtime: search_runtime + route_runtime,
                })
            })
            .map_err(CompassAppError::SearchError)
    }

    /// helper function for accessing the TraversalModel
    ///
    /// example:
    ///
    /// let search_app: SearchApp = ...;
    /// let reference = search_app.get_traversal_model_reference();
    /// let traversal_model = reference.read();
    /// // do things with TraversalModel
    pub fn get_traversal_model_service_reference(
        &self,
    ) -> Arc<ExecutorReadOnlyLock<Arc<dyn TraversalModelService>>> {
        Arc::new(self.traversal_model_service.read_only())
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
        query: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModel>, CompassAppError> {
        let tm = self
            .traversal_model_service
            .read_only()
            .read()
            .map_err(|e| CompassAppError::ReadOnlyPoisonError(e.to_string()))?
            .build(query)?;
        Ok(tm)
    }
}
