use super::search_app_result::SearchAppResult;
use crate::{
    app::compass::{
        compass_app_error::CompassAppError,
        config::cost_model::cost_model_service::CostModelService,
    },
    plugin::input::input_json_extensions::InputJsonExtensions,
};
use chrono::Local;
use routee_compass_core::{
    algorithm::search::{backtrack, search_algorithm::SearchAlgorithm},
    model::{
        cost::cost_model::CostModel,
        frontier::frontier_model_service::FrontierModelService,
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
    cost_model_service: Arc<DriverReadOnlyLock<CostModelService>>,
    frontier_model_service: Arc<DriverReadOnlyLock<Arc<dyn FrontierModelService>>>,
    termination_model: Arc<DriverReadOnlyLock<TerminationModel>>,
}

impl SearchApp {
    /// builds a new SearchApp from the required components.
    /// handles all of the specialized boxing that allows for simple parallelization.
    pub fn new(
        search_algorithm: SearchAlgorithm,
        graph: Graph,
        traversal_model_service: Arc<dyn TraversalModelService>,
        utility_model_service: CostModelService,
        frontier_model_service: Arc<dyn FrontierModelService>,
        termination_model: TerminationModel,
    ) -> Self {
        let graph = Arc::new(DriverReadOnlyLock::new(graph));
        let traversal_model_service = Arc::new(DriverReadOnlyLock::new(traversal_model_service));
        let utility_model_service = Arc::new(DriverReadOnlyLock::new(utility_model_service));
        let frontier_model_service = Arc::new(DriverReadOnlyLock::new(frontier_model_service));
        let termination_model = Arc::new(DriverReadOnlyLock::new(termination_model));
        SearchApp {
            search_algorithm,
            graph,
            traversal_model_service,
            cost_model_service: utility_model_service,
            frontier_model_service,
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
        let state_variable_names = tm_inner.state_variable_names();

        let um_inner = self
            .cost_model_service
            .read_only()
            .read()
            .map_err(|e| CompassAppError::ReadOnlyPoisonError(e.to_string()))?
            .build(query, &state_variable_names)?;

        let fm_inner = self
            .frontier_model_service
            .read_only()
            .read()
            .map_err(|e| CompassAppError::ReadOnlyPoisonError(e.to_string()))?
            .build(query)?;

        let rm_inner = Arc::new(self.termination_model.read_only());
        self.search_algorithm
            .run_vertex_oriented(o, d, dg_inner, tm_inner, um_inner, fm_inner, rm_inner)
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
        let state_variable_names = tm_inner.state_variable_names();

        let um_inner = self
            .cost_model_service
            .read_only()
            .read()
            .map_err(|e| CompassAppError::ReadOnlyPoisonError(e.to_string()))?
            .build(query, &state_variable_names)?;

        let fm_inner = self
            .frontier_model_service
            .read_only()
            .read()
            .map_err(|e| CompassAppError::ReadOnlyPoisonError(e.to_string()))?
            .build(query)?;

        let rm_inner = Arc::new(self.termination_model.read_only());
        self.search_algorithm
            .run_edge_oriented(
                o,
                d,
                dg_inner_search,
                tm_inner,
                um_inner,
                fm_inner,
                rm_inner,
            )
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
    pub fn build_traversal_model(
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

    /// helper function for building an instance of a CostModel
    ///
    /// example:
    ///
    /// let search_app: SearchApp = ...;
    /// let reference = search_app.get_traversal_model_reference();
    /// let traversal_model = reference.read();
    /// // do things with TraversalModel
    pub fn build_cost_model(
        &self,
        query: &serde_json::Value,
    ) -> Result<CostModel, CompassAppError> {
        let tm = self.build_traversal_model(query)?;
        let state_variable_names = tm.state_variable_names();
        let cm = self
            .cost_model_service
            .read_only()
            .read()
            .map_err(|e| CompassAppError::ReadOnlyPoisonError(e.to_string()))?
            .build(query, &state_variable_names)?;
        Ok(cm)
    }

    /// helper function for building an instance of a CostModel
    /// using an already-constructed traversal model (which is an
    /// upstream dependency of building a cost model).
    ///
    /// example:
    ///
    /// let search_app: SearchApp = ...;
    /// let reference = search_app.get_traversal_model_reference();
    /// let traversal_model = reference.read();
    /// // do things with TraversalModel
    pub fn build_cost_model_for_traversal_model(
        &self,
        query: &serde_json::Value,
        tm: Arc<dyn TraversalModel>,
    ) -> Result<CostModel, CompassAppError> {
        let state_variable_names = tm.state_variable_names();
        let cm = self
            .cost_model_service
            .read_only()
            .read()
            .map_err(|e| CompassAppError::ReadOnlyPoisonError(e.to_string()))?
            .build(query, &state_variable_names)?;
        Ok(cm)
    }

    pub fn get_graph_reference(&self) -> Arc<ExecutorReadOnlyLock<Graph>> {
        Arc::new(self.graph.read_only())
    }
}
