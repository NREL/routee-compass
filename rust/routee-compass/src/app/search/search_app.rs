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
    algorithm::search::{
        backtrack, search_algorithm::SearchAlgorithm, search_error::SearchError,
        search_instance::SearchInstance,
    },
    model::{
        access::access_model_service::AccessModelService,
        frontier::frontier_model_service::FrontierModelService, road_network::graph::Graph,
        state::state_model::StateModel, termination::termination_model::TerminationModel,
        traversal::traversal_model_service::TraversalModelService,
    },
};
use std::sync::Arc;
use std::time;

/// a configured and loaded application to execute searches.
pub struct SearchApp {
    pub search_algorithm: SearchAlgorithm,
    pub directed_graph: Arc<Graph>,
    pub state_model: Arc<StateModel>,
    pub traversal_model_service: Arc<dyn TraversalModelService>,
    pub access_model_service: Arc<dyn AccessModelService>,
    pub cost_model_service: Arc<CostModelService>,
    pub frontier_model_service: Arc<dyn FrontierModelService>,
    pub termination_model: Arc<TerminationModel>,
}

impl SearchApp {
    /// builds a new SearchApp from the required components.
    /// handles all of the specialized boxing that allows for simple parallelization.
    pub fn new(
        search_algorithm: SearchAlgorithm,
        graph: Graph,
        state_model: Arc<StateModel>,
        traversal_model_service: Arc<dyn TraversalModelService>,
        access_model_service: Arc<dyn AccessModelService>,
        cost_model_service: CostModelService,
        frontier_model_service: Arc<dyn FrontierModelService>,
        termination_model: TerminationModel,
    ) -> Self {
        SearchApp {
            search_algorithm,
            directed_graph: Arc::new(graph),
            state_model,
            traversal_model_service,
            access_model_service,
            cost_model_service: Arc::new(cost_model_service),
            frontier_model_service,
            termination_model: Arc::new(termination_model),
        }
    }

    /// runs a single vertex oriented query
    ///
    pub fn run_vertex_oriented(
        &self,
        query: &serde_json::Value,
    ) -> Result<(SearchAppResult, SearchInstance), CompassAppError> {
        let o = query
            .get_origin_vertex()
            .map_err(CompassAppError::PluginError)?;
        let d = query
            .get_destination_vertex()
            .map_err(CompassAppError::PluginError)?;
        let search_start_time = Local::now();

        let search_instance = self.build_search_instance(query)?;
        self.search_algorithm
            .run_vertex_oriented(o, d, &search_instance)
            .and_then(|search_result| {
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
                    Some(dest) => backtrack::vertex_oriented_route(o, dest, &search_result.tree)?,
                };
                let route_end_time = Local::now();
                let route_runtime = (route_end_time - route_start_time)
                    .to_std()
                    .unwrap_or(time::Duration::ZERO);
                log::debug!(
                    "Route Computed in {:?} miliseconds",
                    route_runtime.as_millis()
                );
                let result = SearchAppResult {
                    route,
                    tree: search_result.tree,
                    search_executed_time: search_start_time.to_rfc3339(),
                    algorithm_runtime: search_runtime,
                    route_runtime,
                    search_app_runtime: search_runtime + route_runtime,
                    iterations: search_result.iterations,
                };
                Ok((result, search_instance))
            })
            .map_err(CompassAppError::SearchError)
    }

    ///
    /// runs a single edge oriented query
    ///
    pub fn run_edge_oriented(
        &self,
        query: &serde_json::Value,
    ) -> Result<(SearchAppResult, SearchInstance), CompassAppError> {
        let o = query
            .get_origin_edge()
            .map_err(CompassAppError::PluginError)?;
        let d = query
            .get_destination_edge()
            .map_err(CompassAppError::PluginError)?;
        let search_start_time = Local::now();
        let search_instance = self.build_search_instance(query)?;
        self.search_algorithm
            .run_edge_oriented(o, d, &search_instance)
            .and_then(|search_result| {
                let search_end_time = Local::now();
                let route_start_time = Local::now();
                let route = match d {
                    None => vec![],
                    Some(dest) => backtrack::edge_oriented_route(
                        o,
                        dest,
                        &search_result.tree,
                        search_instance.directed_graph.clone(),
                    )?,
                };
                let route_end_time = Local::now();
                let search_runtime = (search_end_time - search_start_time)
                    .to_std()
                    .unwrap_or(time::Duration::ZERO);
                let route_runtime = (route_end_time - route_start_time)
                    .to_std()
                    .unwrap_or(time::Duration::ZERO);
                let result = SearchAppResult {
                    route,
                    tree: search_result.tree,
                    search_executed_time: search_start_time.to_rfc3339(),
                    algorithm_runtime: search_runtime,
                    route_runtime,
                    search_app_runtime: search_runtime + route_runtime,
                    iterations: search_result.iterations,
                };
                Ok((result, search_instance))
            })
            .map_err(CompassAppError::SearchError)
    }

    pub fn build_search_instance(
        &self,
        query: &serde_json::Value,
    ) -> Result<SearchInstance, SearchError> {
        let traversal_model = self.traversal_model_service.build(query)?;
        let access_model = self.access_model_service.build(query)?;

        let mut added_features = traversal_model.state_features();
        added_features.extend(access_model.state_features());
        let state_model = Arc::new(self.state_model.extend(added_features)?);

        let cost_model = self
            .cost_model_service
            .build(query, state_model.clone())
            .map_err(|e| SearchError::BuildError(e.to_string()))?;
        let frontier_model = self
            .frontier_model_service
            .build(query, state_model.clone())?;

        let search_assets = SearchInstance {
            directed_graph: self.directed_graph.clone(),
            state_model,
            traversal_model,
            access_model,
            cost_model,
            frontier_model,
            termination_model: self.termination_model.clone(),
        };

        Ok(search_assets)
    }
}
