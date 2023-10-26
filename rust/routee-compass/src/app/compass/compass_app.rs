use super::config::compass_app_builder::CompassAppBuilder;
use crate::{
    app::{
        compass::{
            compass_configuration_field::CompassConfigurationField,
            config::termination_model_builder::TerminationModelBuilder,
        },
        compass_app_error::CompassAppError,
        search::{search_app::SearchApp, search_app_result::SearchAppResult},
    },
    plugin::{
        input::input_plugin::InputPlugin, output::output_plugin::OutputPlugin,
        plugin_error::PluginError,
    },
};
use chrono::{Duration, Local};
use config::Config;
use itertools::{Either, Itertools};
use rayon::{current_num_threads, prelude::*};
use routee_compass_core::{
    algorithm::search::search_algorithm::SearchAlgorithm,
    model::{
        cost::cost::Cost,
        graph::{graph::Graph, graph_config::GraphConfig},
    },
    util::duration_extension::DurationExtension,
};
use std::path::PathBuf;

/// Instance of RouteE Compass as an application.
/// When constructed, it holds
///   - the core search application which performs parallel path search
///   - the input plugins for query pre-processing
///   - the output plugins for query post-processing
///
/// A CompassApp instance provides the high-level API for building and
/// running RouteE Compass.
pub struct CompassApp {
    pub search_app: SearchApp,
    pub input_plugins: Vec<Box<dyn InputPlugin>>,
    pub output_plugins: Vec<Box<dyn OutputPlugin>>,
    pub parallelism: usize,
}

impl TryFrom<PathBuf> for CompassApp {
    type Error = CompassAppError;

    /// Builds a CompassApp from a configuration filepath, using the default CompassAppBuilder.
    /// Builds all components such as the DirectedGraph, TraversalModel, and SearchAlgorithm.
    /// Also builds the input and output plugins.
    /// Returns a persistent application that can run user queries in parallel.
    ///
    /// # Arguments
    ///
    /// * `conf_file` - path to a configuration TOML file
    ///
    /// # Returns
    ///
    /// * an instance of [`CompassApp`], or an error if load failed.
    fn try_from(conf_file: PathBuf) -> Result<Self, Self::Error> {
        let default_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("app")
            .join("compass")
            .join("config")
            .join("config.default.toml");

        let config = Config::builder()
            .add_source(config::File::from(default_file))
            .add_source(config::File::from(conf_file))
            .build()
            .map_err(CompassAppError::ConfigError)?;
        log::info!("Config: {:?}", config);
        let builder = CompassAppBuilder::default();
        let compass_app = CompassApp::try_from((&config, &builder))?;
        return Ok(compass_app);
    }
}

impl TryFrom<(&Config, &CompassAppBuilder)> for CompassApp {
    type Error = CompassAppError;

    /// Builds a CompassApp from configuration and a (possibly customized) CompassAppBuilder.
    /// Builds all modules such as the DirectedGraph, TraversalModel, and SearchAlgorithm.
    /// Also builds the input and output plugins.
    /// Returns a persistent application that can run user queries in parallel.
    ///
    /// This is the extension API for building [`CompassApp`] instances.
    /// In application, the user becomes responsible for
    ///   -
    ///
    /// # Arguments
    ///
    /// * `pair` - a tuple containing a config object (such as a parsed TOML file) and
    ///            a [`super::config::compass_app_builder::CompassAppBuilder`] instance
    ///
    /// # Returns
    ///
    /// * an instance of [`CompassApp`], or an error if load failed.
    fn try_from(pair: (&Config, &CompassAppBuilder)) -> Result<Self, Self::Error> {
        let (config, builder) = pair;

        let alg_params =
            config.get::<serde_json::Value>(CompassConfigurationField::Algorithm.to_str())?;
        let search_algorithm = SearchAlgorithm::try_from(&alg_params)?;

        // build traversal model
        let traversal_start = Local::now();
        let traversal_params =
            config.get::<serde_json::Value>(CompassConfigurationField::Traversal.to_str())?;
        let traversal_model_service = builder.build_traversal_model_service(&traversal_params)?;
        let traversal_duration = (Local::now() - traversal_start)
            .to_std()
            .map_err(|e| CompassAppError::InternalError(e.to_string()))?;
        log::info!(
            "finished reading traversal model with duration {}",
            traversal_duration.hhmmss()
        );

        // build frontier model
        let frontier_start = Local::now();
        let frontier_params =
            config.get::<serde_json::Value>(CompassConfigurationField::Frontier.to_str())?;
        let frontier_model = builder.build_frontier_model(frontier_params)?;
        let frontier_duration = (Local::now() - frontier_start)
            .to_std()
            .map_err(|e| CompassAppError::InternalError(e.to_string()))?;
        log::info!(
            "finished reading frontier model with duration {}",
            frontier_duration.hhmmss()
        );

        // build termination model
        let termination_model_json =
            config.get::<serde_json::Value>(CompassConfigurationField::Termination.to_str())?;
        let termination_model = TerminationModelBuilder::build(&termination_model_json)?;

        // build graph
        let graph_start = Local::now();
        let graph_conf = &config
            .get::<GraphConfig>(CompassConfigurationField::Graph.to_str())
            .map_err(CompassAppError::ConfigError)?;
        let graph = Graph::try_from(graph_conf)?;
        let graph_duration = (Local::now() - graph_start)
            .to_std()
            .map_err(|e| CompassAppError::InternalError(e.to_string()))?;
        log::info!(
            "finished reading graph with duration {}",
            graph_duration.hhmmss()
        );

        // build search app
        let search_app_start = Local::now();
        let parallelism = config.get::<usize>(CompassConfigurationField::Parallelism.to_str())?;
        let search_app: SearchApp = SearchApp::new(
            search_algorithm,
            graph,
            traversal_model_service,
            frontier_model,
            termination_model,
        );
        let search_app_duration = to_std(Local::now() - search_app_start)?;
        log::info!(
            "finished building search app with duration {}",
            search_app_duration.hhmmss()
        );

        // build plugins
        let plugins_start = Local::now();
        let plugins_config =
            config.get::<serde_json::Value>(CompassConfigurationField::Plugins.to_str())?;

        let input_plugins = builder.build_input_plugins(plugins_config.clone())?;
        let output_plugins = builder.build_output_plugins(plugins_config.clone())?;

        let plugins_duration = to_std(Local::now() - plugins_start)?;
        log::info!(
            "finished loading plugins with duration {}",
            plugins_duration.hhmmss()
        );

        return Ok(CompassApp {
            search_app,
            input_plugins,
            output_plugins,
            parallelism,
        });
    }
}

impl CompassApp {
    /// runs a set of queries via this instance of CompassApp. this
    ///   1. processes each input query based on the InputPlugins
    ///   2. runs the search algorithm with each query via SearchApp
    ///   3. processes each output based on the OutputPlugins
    ///   4. returns the JSON response
    ///
    /// only  errors should cause CompassApp to halt. if there are
    /// errors due to the user, they should be propagated along into the output
    /// JSON in an error format along with the request.
    pub fn run(
        &self,
        queries: Vec<serde_json::Value>,
    ) -> Result<Vec<serde_json::Value>, CompassAppError> {
        // input plugins need to be flattened, and queries that fail input processing need to be
        // returned at the end.
        let (input_bundles, input_error_responses): (
            Vec<Vec<serde_json::Value>>,
            Vec<serde_json::Value>,
        ) = queries
            .iter()
            .map(|q| apply_input_plugins(&q, &self.input_plugins))
            .partition_map(|r| match r {
                Ok(values) => Either::Left(values),
                Err(error_response) => Either::Right(error_response),
            });
        let input_queries: Vec<serde_json::Value> = input_bundles.into_iter().flatten().collect();

        // run parallel searches using a rayon thread pool
        let chunk_size = (input_queries.len() as f64 / self.parallelism as f64).ceil() as usize;
        log::info!(
            "creating {} parallel batches across {} threads to run queries with chunk size {}",
            self.parallelism,
            current_num_threads(),
            chunk_size
        );

        let run_query_result = input_queries
            .par_chunks(chunk_size)
            .map(|queries| {
                queries
                    .iter()
                    .map(|q| self.run_single_query(q.clone()))
                    .collect::<Result<Vec<serde_json::Value>, CompassAppError>>()
            })
            .collect::<Result<Vec<Vec<serde_json::Value>>, CompassAppError>>()?;

        let run_result = run_query_result
            .into_iter()
            .flatten()
            .chain(input_error_responses)
            .collect();

        return Ok(run_result);
    }

    /// Helper function that runs CompassApp on a single query.
    /// It is assumed that all pre-processing from InputPlugins have been applied.
    /// This function runs a vertex-oriented search and feeds the result into the
    /// OutputPlugins for post-processing, returning the result as JSON.
    ///
    /// # Arguments
    ///
    /// * `query` - a single search query that has been processed by InputPlugins
    ///
    /// # Returns
    ///
    /// * The result of the search and post-processing as a JSON object, or, an error
    pub fn run_single_query(
        &self,
        query: serde_json::Value,
    ) -> Result<serde_json::Value, CompassAppError> {
        let search_result = self.search_app.run_vertex_oriented(&query);
        let output = apply_output_processing(
            (&query, search_result),
            &self.search_app,
            &self.output_plugins,
        );
        Ok(output)
    }
}

/// helper for handling conversion from Chrono Duration to std Duration
fn to_std(dur: Duration) -> Result<std::time::Duration, CompassAppError> {
    dur.to_std().map_err(|e| {
        CompassAppError::InternalError(format!(
            "unexpected internal error mapping chrono duration to std duration: {}",
            e.to_string()
        ))
    })
}

/// helper that applies the input plugins to a query, returning the result(s) or an error if failed
pub fn apply_input_plugins(
    query: &serde_json::Value,
    plugins: &Vec<Box<dyn InputPlugin>>,
) -> Result<Vec<serde_json::Value>, serde_json::Value> {
    let init = Ok(vec![query.clone()]);
    let result = plugins
        .iter()
        .fold(init, |acc, p| {
            acc.and_then(|outer| {
                outer
                    .iter()
                    .map(|q| p.process(q))
                    .collect::<Result<Vec<_>, PluginError>>()
                    .map(|inner| {
                        inner
                            .into_iter()
                            .flatten()
                            .collect::<Vec<serde_json::Value>>()
                    })
            })
        })
        .map_err(|e| {
            serde_json::json!({
                "request": query,
                "error": e.to_string()
            })
        })?;
    return Ok(result);
}

// helper that applies the output processing. this includes
// 1. summarizing from the TraversalModel
// 2. applying the output plugins
pub fn apply_output_processing(
    response_data: (&serde_json::Value, Result<SearchAppResult, CompassAppError>),
    search_app: &SearchApp,
    output_plugins: &Vec<Box<dyn OutputPlugin>>,
) -> serde_json::Value {
    let (req, res) = response_data;
    match res {
        Err(e) => {
            let error_output = serde_json::json!({
                "request": req,
                "error": e.to_string()
            });
            error_output
        }
        Ok(result) => {
            // should be moved into TraversalModel::summary, queries requesting
            // min spanning tree result will not have an acc_cost.
            let mut acc_cost = Cost::ZERO;
            for traversal in result.route.clone() {
                let cost = traversal.edge_cost();
                acc_cost = acc_cost + cost;
            }

            log::debug!(
                "completed route for request {}: {} links, {} tree size",
                req,
                result.route.len(),
                result.tree.len()
            );

            // should be moved into TraversalModel::summary same reason as above
            let route = result.route.to_vec();
            let last_edge_traversal = match route.last() {
                None => {
                    return serde_json::json!({
                        "request": req,
                        "error": "route was empty"
                    });
                }
                Some(et) => et,
            };

            let tmodel = match search_app.get_traversal_model_reference(req) {
                Err(e) => {
                    return serde_json::json!({
                        "request": req,
                        "error": e.to_string()
                    })
                }
                Ok(tmodel) => tmodel,
            };

            let init_output = serde_json::json!({
                "request": req,
                "search_executed_time": result.search_start_time.to_rfc3339(),
                "search_runtime": result.search_runtime.hhmmss(),
                "route_runtime": result.route_runtime.hhmmss(),
                "total_runtime": result.total_runtime.hhmmss(),
                "route_edge_count": result.route.len(),
                "tree_edge_count": result.tree.len(),
                "traversal_summary": tmodel.serialize_state_with_info(&last_edge_traversal.result_state),
            });
            let init_acc: Result<serde_json::Value, PluginError> = Ok(init_output);
            let json_result = output_plugins
                .iter()
                .fold(init_acc, move |acc, plugin| match acc {
                    Err(e) => Err(e),
                    Ok(json) => plugin.process(&json, Ok(&result)),
                })
                .map_err(CompassAppError::PluginError);
            match json_result {
                Err(e) => {
                    serde_json::json!({
                        "request": req,
                        "error": e.to_string()
                    })
                }
                Ok(json) => json,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::CompassApp;

    #[test]
    fn test_speeds() {
        // rust runs test and debug at different locations, which breaks the URLs
        // written in the referenced TOML files. here's a quick fix
        // turnaround that doesn't leak into anyone's VS Code settings.json files
        // see https://github.com/rust-lang/rust-analyzer/issues/4705 for discussion
        let conf_file_test = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("app")
            .join("compass")
            .join("test")
            .join("speeds_test")
            .join("speeds_test.toml");

        let conf_file_debug = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("app")
            .join("compass")
            .join("test")
            .join("speeds_test")
            .join("speeds_debug.toml");

        let app = CompassApp::try_from(conf_file_test)
            .or(CompassApp::try_from(conf_file_debug))
            .unwrap();
        let query = serde_json::json!({
            "origin_vertex": 0,
            "destination_vertex": 2
        });
        let result = app.run(vec![query]).unwrap();
        let edge_ids = result[0].get("edge_id_list").unwrap();
        // path [1] is distance-optimal; path [0, 2] is time-optimal
        let expected = serde_json::json!(vec![0, 2]);
        assert_eq!(edge_ids, &expected);
    }

    // #[test]
    // fn test_energy() {
    //     // rust runs test and debug at different locations, which breaks the URLs
    //     // written in the referenced TOML files. here's a quick fix
    //     // turnaround that doesn't leak into anyone's VS Code settings.json files
    //     // see https://github.com/rust-lang/rust-analyzer/issues/4705 for discussion
    //     let conf_file_test = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //         .join("src")
    //         .join("app")
    //         .join("compass")
    //         .join("test")
    //         .join("energy_test")
    //         .join("energy_test.toml");

    //     let conf_file_debug = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //         .join("src")
    //         .join("app")
    //         .join("compass")
    //         .join("test")
    //         .join("energy_test")
    //         .join("energy_debug.toml");

    //     let app = CompassApp::try_from(conf_file_test)
    //         .or(CompassApp::try_from(conf_file_debug))
    //         .unwrap();
    //     let query = serde_json::json!({
    //         "origin_vertex": 0,
    //         "destination_vertex": 2
    //     });
    //     let result = app.run(vec![query]).unwrap();
    //     let edge_ids = result[0].get("edge_id_list").unwrap();
    //     // path [1] is distance-optimal; path [0, 2] is time-optimal
    //     let expected = serde_json::json!(vec![0, 2]);
    //     assert_eq!(edge_ids, &expected);
    // }
}
