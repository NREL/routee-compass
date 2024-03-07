use super::search_app_result::SearchAppResult;
use crate::{
    app::compass::{
        compass_app_error::CompassAppError,
        config::{
            compass_configuration_field::CompassConfigurationField,
            config_json_extension::ConfigJsonExtensions,
            cost_model::{
                cost_model_builder::CostModelBuilder, cost_model_service::CostModelService,
            },
            graph_builder::DefaultGraphBuilder,
            termination_model_builder::TerminationModelBuilder,
        },
    },
    plugin::input::input_json_extensions::InputJsonExtensions,
};
use chrono::Local;
use routee_compass_core::{
    algorithm::search::{
        self, backtrack, search_algorithm::SearchAlgorithm, search_error::SearchError,
        search_instance::SearchInstance,
    },
    model::{
        cost::cost_model::CostModel,
        frontier::frontier_model_service::FrontierModelService,
        road_network::graph::Graph,
        state::state_model::StateModel,
        termination::termination_model::TerminationModel,
        traversal::{
            traversal_model::TraversalModel, traversal_model_service::TraversalModelService,
        },
    },
    util::{
        duration_extension::DurationExtension,
        read_only_lock::{DriverReadOnlyLock, ExecutorReadOnlyLock},
    },
};
use std::time;
use std::{path::PathBuf, sync::Arc};

pub struct SearchApp {
    pub search_algorithm: SearchAlgorithm,
    pub directed_graph: Arc<Graph>,
    pub state_model: Arc<StateModel>,
    pub traversal_model_service: Arc<dyn TraversalModelService>,
    pub cost_model_service: Arc<CostModelService>,
    pub frontier_model_service: Arc<dyn FrontierModelService>,
    pub termination_model: Arc<TerminationModel>,
}

// impl TryFrom<&serde_json::Value> for SearchApp {
//     type Error = SearchError;

//     fn try_from(config: &serde_json::Value) -> Result<Self, Self::Error> {
//         let alg_params = config.get_config_section(CompassConfigurationField::Algorithm)?;
//         let search_algorithm = SearchAlgorithm::try_from(&alg_params)?;

//         // build traversal model
//         let traversal_start = Local::now();
//         let traversal_params = config.get_config_section(CompassConfigurationField::Traversal)?;
//         let traversal_model_service = builder.build_traversal_model_service(&traversal_params)?;
//         let traversal_duration = (Local::now() - traversal_start)
//             .to_std()
//             .map_err(|e| CompassAppError::InternalError(e.to_string()))?;
//         log::info!(
//             "finished reading traversal model with duration {}",
//             traversal_duration.hhmmss()
//         );

//         // build utility model
//         let cost_params = config.get_config_section(CompassConfigurationField::Cost)?;
//         let cost_model_service = CostModelBuilder {}.build(&cost_params)?;

//         // build frontier model
//         let frontier_start = Local::now();
//         let frontier_params = config.get_config_section(CompassConfigurationField::Frontier)?;

//         let frontier_model_service = builder.build_frontier_model_service(&frontier_params)?;

//         let frontier_duration = (Local::now() - frontier_start)
//             .to_std()
//             .map_err(|e| CompassAppError::InternalError(e.to_string()))?;
//         log::info!(
//             "finished reading frontier model with duration {}",
//             frontier_duration.hhmmss()
//         );

//         // build termination model
//         let termination_model_json =
//             config.get_config_section(CompassConfigurationField::Termination)?;
//         let termination_model = TerminationModelBuilder::build(&termination_model_json, None)?;

//         // build graph
//         let graph_start = Local::now();
//         let graph_params = config.get_config_section(CompassConfigurationField::Graph)?;
//         let graph = DefaultGraphBuilder::build(&graph_params)?;
//         let graph_duration = (Local::now() - graph_start)
//             .to_std()
//             .map_err(|e| CompassAppError::InternalError(e.to_string()))?;
//         log::info!(
//             "finished reading graph with duration {}",
//             graph_duration.hhmmss()
//         );

//         let graph_bytes = allocative::size_of_unique_allocated_data(&graph);
//         log::info!("graph size: {} GB", graph_bytes as f64 / 1e9);

//         #[cfg(debug_assertions)]
//         {
//             use std::io::Write;

//             log::debug!("Building flamegraph for graph memory usage..");

//             let mut flamegraph = allocative::FlameGraphBuilder::default();
//             flamegraph.visit_root(&graph);
//             let output = flamegraph.finish_and_write_flame_graph();

//             let outdir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
//                 .join("..")
//                 .join("target")
//                 .join("flamegraph");

//             if !outdir.exists() {
//                 std::fs::create_dir(&outdir).unwrap();
//             }

//             let outfile = outdir.join("graph_memory_flamegraph.out");

//             log::debug!("writing graph flamegraph to {:?}", outfile);

//             let mut output_file = std::fs::File::create(outfile).unwrap();
//             output_file.write_all(output.as_bytes()).unwrap();
//         }

//         // build search app
//         let search_app: SearchApp = SearchApp::new(
//             search_algorithm,
//             graph,
//             traversal_model_service,
//             cost_model_service,
//             frontier_model_service,
//             termination_model,
//         );

//         Ok(search_app)
//     }
// }

impl SearchApp {
    /// builds a new SearchApp from the required components.
    /// handles all of the specialized boxing that allows for simple parallelization.
    pub fn new(
        search_algorithm: SearchAlgorithm,
        graph: Graph,
        state_model: Arc<StateModel>,
        traversal_model_service: Arc<dyn TraversalModelService>,
        cost_model_service: CostModelService,
        frontier_model_service: Arc<dyn FrontierModelService>,
        termination_model: TerminationModel,
    ) -> Self {
        SearchApp {
            search_algorithm,
            directed_graph: Arc::new(graph),
            state_model,
            traversal_model_service,
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
        let traversal_model = self
            .traversal_model_service
            .build(query, self.state_model.clone())?;
        let cost_model = self
            .cost_model_service
            .build(query, self.state_model.clone())
            .map_err(|e| SearchError::BuildError(e.to_string()))?;
        let frontier_model = self
            .frontier_model_service
            .build(query, self.state_model.clone())?;
        let state_model = self.state_model.extend(traversal_model.state_features())?;

        let search_assets = SearchInstance {
            directed_graph: self.directed_graph.clone(),
            state_model: Arc::new(state_model),
            traversal_model,
            cost_model,
            frontier_model,
            termination_model: self.termination_model.clone(),
        };

        Ok(search_assets)
    }

    // /// helper function for accessing the TraversalModel
    // ///
    // /// example:
    // ///
    // /// let search_app: SearchApp = ...;
    // /// let reference = search_app.get_traversal_model_reference();
    // /// let traversal_model = reference.read();
    // /// // do things with TraversalModel
    // pub fn get_traversal_model_service_reference(
    //     &self,
    // ) -> Arc<ExecutorReadOnlyLock<Arc<dyn TraversalModelService>>> {
    //     Arc::new(self.traversal_model_service.read_only())
    // }

    // /// helper function for accessing the TraversalModel
    // ///
    // /// example:
    // ///
    // /// let search_app: SearchApp = ...;
    // /// let reference = search_app.get_traversal_model_reference();
    // /// let traversal_model = reference.read();
    // /// // do things with TraversalModel
    // pub fn build_traversal_model(
    //     &self,
    //     query: &serde_json::Value,
    // ) -> Result<Arc<dyn TraversalModel>, CompassAppError> {
    //     let tm = self
    //         .traversal_model_service
    //         .build(query, self.state_model.clone())?;
    //     Ok(tm)
    // }

    // /// helper function for building an instance of a CostModel
    // ///
    // /// example:
    // ///
    // /// let search_app: SearchApp = ...;
    // /// let reference = search_app.get_traversal_model_reference();
    // /// let traversal_model = reference.read();
    // /// // do things with TraversalModel
    // pub fn build_cost_model(
    //     &self,
    //     query: &serde_json::Value,
    // ) -> Result<CostModel, CompassAppError> {
    //     let tm = self.build_traversal_model(query)?;
    //     let cm = self
    //         .cost_model_service
    //         .read_only()
    //         .read()
    //         .map_err(|e| CompassAppError::ReadOnlyPoisonError(e.to_string()))?
    //         .build(query, self.state_model.clone())?;
    //     Ok(cm)
    // }

    // /// helper function for building an instance of a CostModel
    // /// using an already-constructed traversal model (which is an
    // /// upstream dependency of building a cost model).
    // ///
    // /// example:
    // ///
    // /// let search_app: SearchApp = ...;
    // /// let reference = search_app.get_traversal_model_reference();
    // /// let traversal_model = reference.read();
    // /// // do things with TraversalModel
    // pub fn build_cost_model_for_traversal_model(
    //     &self,
    //     query: &serde_json::Value,
    //     tm: Arc<dyn TraversalModel>,
    // ) -> Result<CostModel, CompassAppError> {
    //     let cm = self
    //         .cost_model_service
    //         .read_only()
    //         .read()
    //         .map_err(|e| CompassAppError::ReadOnlyPoisonError(e.to_string()))?
    //         .build(query, self.state_model.clone())?;
    //     Ok(cm)
    // }

    // pub fn get_graph_reference(&self) -> Arc<ExecutorReadOnlyLock<Graph>> {
    //     Arc::new(self.directed_graph.read_only())
    // }
}
