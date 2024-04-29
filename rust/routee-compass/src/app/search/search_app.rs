use super::{search_app_ops, search_app_result::SearchAppResult};
use crate::{
    app::compass::{
        compass_app_error::CompassAppError,
        config::cost_model::cost_model_service::CostModelService,
        search_orientation::SearchOrientation,
    },
    plugin::input::input_json_extensions::InputJsonExtensions,
};
use chrono::Local;
use routee_compass_core::{
    algorithm::search::{
        direction::Direction, search_algorithm::SearchAlgorithm,
        search_algorithm_result::SearchAlgorithmResult, search_error::SearchError,
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
    #[allow(clippy::too_many_arguments)]
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

    /// main interface for running search. takes a user query and some configured
    /// search orientation. builds the instance of the search assets and then executes
    /// a search. if a destination is set on the query, then the route is computed.
    /// if the algorithm produces more than one route, then the result contains each route.
    /// the SearchAlgorithm determines the order and number of routes and trees in the result.
    ///
    /// # Arguments
    ///
    /// * `query` - a JSON search query provided by the user
    /// * `search_orientation` - whether to orient by vertex or edge
    ///
    /// # Results
    ///
    /// The complete set of trees, routes, and search assets for this run.
    pub fn run(
        &self,
        query: &serde_json::Value,
        search_orientation: &SearchOrientation,
    ) -> Result<(SearchAppResult, SearchInstance), CompassAppError> {
        let search_start_time = Local::now();
        let (results, si) = match search_orientation {
            SearchOrientation::Vertex => self.run_vertex_oriented(query),
            SearchOrientation::Edge => self.run_edge_oriented(query),
        }?;

        let search_end_time = Local::now();
        let search_runtime = (search_end_time - search_start_time)
            .to_std()
            .unwrap_or(time::Duration::ZERO);

        log::debug!(
            "Search Completed in {:?} miliseconds",
            search_runtime.as_millis()
        );

        let result = SearchAppResult {
            routes: results.routes,
            trees: results.trees,
            search_executed_time: search_start_time.to_rfc3339(),
            search_runtime,
            iterations: results.iterations,
        };

        Ok((result, si))
    }

    pub fn run_vertex_oriented(
        &self,
        query: &serde_json::Value,
    ) -> Result<(SearchAlgorithmResult, SearchInstance), CompassAppError> {
        let o = query
            .get_origin_vertex()
            .map_err(CompassAppError::PluginError)?;
        let d = query
            .get_destination_vertex()
            .map_err(CompassAppError::PluginError)?;

        let search_instance = self.build_search_instance(query)?;
        self.search_algorithm
            .run_vertex_oriented(o, d, &Direction::Forward, &search_instance)
            .map(|search_result| (search_result, search_instance))
            .map_err(CompassAppError::SearchError)
    }

    pub fn run_edge_oriented(
        &self,
        query: &serde_json::Value,
    ) -> Result<(SearchAlgorithmResult, SearchInstance), CompassAppError> {
        let o = query
            .get_origin_edge()
            .map_err(CompassAppError::PluginError)?;
        let d_opt = query
            .get_destination_edge()
            .map_err(CompassAppError::PluginError)?;
        let search_instance = self.build_search_instance(query)?;
        self.search_algorithm
            .run_edge_oriented(o, d_opt, &Direction::Forward, &search_instance)
            .map(|search_result| (search_result, search_instance))
            .map_err(CompassAppError::SearchError)
    }

    /// builds the assets that will run the search for this query instance.
    ///
    /// # Arguments
    ///
    /// * `query` - the user query initiating this search
    ///
    /// # Results
    ///
    /// The SearchInstance which runs this search query.
    pub fn build_search_instance(
        &self,
        query: &serde_json::Value,
    ) -> Result<SearchInstance, SearchError> {
        let traversal_model = self.traversal_model_service.build(query)?;
        let access_model = self.access_model_service.build(query)?;

        let state_features =
            search_app_ops::collect_features(query, traversal_model.clone(), access_model.clone())?;
        let state_model_instance = self.state_model.extend(state_features)?;
        let state_model = Arc::new(state_model_instance);

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
