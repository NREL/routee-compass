use super::{
    compass_app_ops as ops, config::compass_app_builder::CompassAppBuilder,
    search_orientation::SearchOrientation,
};
use crate::{
    app::{
        compass::{
            compass_app_error::CompassAppError,
            compass_input_field::CompassInputField,
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
use kdam::{Bar, BarExt};
use rayon::{current_num_threads, prelude::*};
use routee_compass_core::{
    algorithm::search::search_algorithm::SearchAlgorithm,
    util::duration_extension::DurationExtension,
};
use serde_json::Value;
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

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
    pub input_plugins: Vec<Arc<dyn InputPlugin>>,
    pub output_plugins: Vec<Arc<dyn OutputPlugin>>,
    pub parallelism: usize,
    pub search_orientation: SearchOrientation,
}

impl TryFrom<&Path> for CompassApp {
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
    fn try_from(conf_file: &Path) -> Result<Self, Self::Error> {
        let config = ops::read_config_from_file(conf_file)?;
        let builder = CompassAppBuilder::default();
        let compass_app = CompassApp::try_from((&config, &builder))?;
        Ok(compass_app)
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

        // Get the root config path so we can resolve paths relative
        // to where the config file is located.
        let root_config_path =
            config.get::<PathBuf>(CompassInputField::ConfigInputFile.to_str())?;

        let config_json = config
            .clone()
            .try_deserialize::<serde_json::Value>()?
            .normalize_file_paths(&"", &root_config_path)?;

        let alg_params = config_json.get_config_section(CompassConfigurationField::Algorithm)?;
        let search_algorithm = SearchAlgorithm::try_from(&alg_params)?;

        // build traversal model
        let traversal_start = Local::now();
        let traversal_params =
            config_json.get_config_section(CompassConfigurationField::Traversal)?;
        let traversal_model_service = builder.build_traversal_model_service(&traversal_params)?;
        let traversal_duration = (Local::now() - traversal_start)
            .to_std()
            .map_err(|e| CompassAppError::InternalError(e.to_string()))?;
        log::info!(
            "finished reading traversal model with duration {}",
            traversal_duration.hhmmss()
        );

        // build utility model
        let utility_params = config_json.get_config_section(CompassConfigurationField::Cost);
        let utility_model_service = match utility_params.ok() {
            None => Ok(CostModelService::default_cost_model()),
            Some(params) => CostModelBuilder {}.build(&params),
        }?;

        // build frontier model
        let frontier_start = Local::now();
        let frontier_params =
            config_json.get_config_section(CompassConfigurationField::Frontier)?;

        let frontier_model_service = builder.build_frontier_model_service(&frontier_params)?;

        let frontier_duration = (Local::now() - frontier_start)
            .to_std()
            .map_err(|e| CompassAppError::InternalError(e.to_string()))?;
        log::info!(
            "finished reading frontier model with duration {}",
            frontier_duration.hhmmss()
        );

        // build termination model
        let termination_model_json =
            config_json.get_config_section(CompassConfigurationField::Termination)?;
        let termination_model = TerminationModelBuilder::build(&termination_model_json, None)?;

        // build graph
        let graph_start = Local::now();
        let graph_params = config_json.get_config_section(CompassConfigurationField::Graph)?;
        let graph = DefaultGraphBuilder::build(&graph_params)?;
        let graph_duration = (Local::now() - graph_start)
            .to_std()
            .map_err(|e| CompassAppError::InternalError(e.to_string()))?;
        log::info!(
            "finished reading graph with duration {}",
            graph_duration.hhmmss()
        );

        // build search app
        let search_app: SearchApp = SearchApp::new(
            search_algorithm,
            graph,
            traversal_model_service,
            utility_model_service,
            frontier_model_service,
            termination_model,
        );

        // build plugins
        let plugins_start = Local::now();
        let plugins_config = config_json.get_config_section(CompassConfigurationField::Plugins)?;

        let input_plugins = builder.build_input_plugins(&plugins_config)?;
        let output_plugins = builder.build_output_plugins(&plugins_config)?;

        let plugins_duration = to_std(Local::now() - plugins_start)?;
        log::info!(
            "finished loading plugins with duration {}",
            plugins_duration.hhmmss()
        );

        // other parameters
        let parallelism = config.get::<usize>(CompassConfigurationField::Parallelism.to_str())?;
        let search_orientation = config
            .get::<SearchOrientation>(CompassConfigurationField::SearchOrientation.to_str())?;

        log::info!(
            "additional parameters - parallelism={}, search orientation={:?}",
            parallelism,
            search_orientation
        );

        Ok(CompassApp {
            search_app,
            input_plugins,
            output_plugins,
            parallelism,
            search_orientation,
        })
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
        let input_pb = Bar::builder()
            .total(queries.len())
            .animation("fillup")
            .desc("input plugins")
            .build()
            .map_err(CompassAppError::UXError)?;
        let input_pb_shared = Arc::new(Mutex::new(input_pb));

        // input plugins need to be flattened, and queries that fail input processing need to be
        // returned at the end.
        let plugin_chunk_size = (queries.len() as f64 / self.parallelism as f64).ceil() as usize;
        let input_plugin_result: (Vec<_>, Vec<_>) = queries
            .par_chunks(plugin_chunk_size)
            .map(|queries| {
                let result: (Vec<Vec<Value>>, Vec<Value>) = queries
                    .iter()
                    .map(|q| {
                        let inner_processed = apply_input_plugins(q, &self.input_plugins);
                        if let Ok(mut pb_local) = input_pb_shared.lock() {
                            let _ = pb_local.update(1);
                        }
                        inner_processed
                    })
                    .partition_map(|r| match r {
                        Ok(values) => Either::Left(values),
                        Err(error_response) => Either::Right(error_response),
                    });

                result
            })
            .unzip();

        println!();

        // unpack input plugin results
        let (processed_inputs_nested, error_inputs_nested) = input_plugin_result;
        let processed_inputs: Vec<Value> = processed_inputs_nested
            .into_iter()
            .flatten()
            .flatten()
            .collect();
        let load_balanced_inputs =
            ops::apply_load_balancing_policy(&processed_inputs, self.parallelism, 1.0)?;
        let error_inputs: Vec<Value> = error_inputs_nested.into_iter().flatten().collect();
        if load_balanced_inputs.is_empty() {
            return Ok(error_inputs);
        }

        log::info!(
            "creating {} parallel batches across {} threads to run queries",
            self.parallelism,
            current_num_threads(),
        );
        let proc_batch_sizes = load_balanced_inputs
            .iter()
            .map(|qs| qs.len())
            .collect::<Vec<_>>();
        log::info!("queries assigned per executor: {:?}", proc_batch_sizes);

        // set up search progress bar
        let num_balanced_inputs = load_balanced_inputs
            .iter()
            .flatten()
            .collect::<Vec<_>>()
            .len();
        let search_pb = Bar::builder()
            .total(num_balanced_inputs)
            .animation("fillup")
            .desc("search")
            .build()
            .map_err(CompassAppError::UXError)?;
        let search_pb_shared = Arc::new(Mutex::new(search_pb));

        // run parallel searches as organized by the (optional) load balancing policy
        // across a thread pool managed by rayon
        let run_query_result = load_balanced_inputs
            .par_iter()
            .map(|queries| {
                queries
                    .iter()
                    .map(|q| {
                        let inner_search = self.run_single_query((**q).clone());
                        if let Ok(mut pb_local) = search_pb_shared.lock() {
                            let _ = pb_local.update(1);
                        }
                        inner_search
                    })
                    .collect::<Result<Vec<Vec<serde_json::Value>>, CompassAppError>>()
            })
            .collect::<Result<Vec<Vec<Vec<serde_json::Value>>>, CompassAppError>>()?;

        let run_result = run_query_result
            .into_iter()
            .flatten()
            .flatten()
            .chain(error_inputs)
            .collect();

        Ok(run_result)
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
    ) -> Result<Vec<serde_json::Value>, CompassAppError> {
        let search_result = match self.search_orientation {
            SearchOrientation::Vertex => self.search_app.run_vertex_oriented(&query),
            SearchOrientation::Edge => self.search_app.run_edge_oriented(&query),
        };
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
            e
        ))
    })
}

/// helper that applies the input plugins to a query, returning the result(s) or an error if failed
pub fn apply_input_plugins(
    query: &serde_json::Value,
    plugins: &[Arc<dyn InputPlugin>],
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
    Ok(result)
}

// helper that applies the output processing. this includes
// 1. summarizing from the TraversalModel
// 2. applying the output plugins
pub fn apply_output_processing(
    response_data: (&serde_json::Value, Result<SearchAppResult, CompassAppError>),
    search_app: &SearchApp,
    output_plugins: &[Arc<dyn OutputPlugin>],
) -> Vec<serde_json::Value> {
    let (req, res) = response_data;

    let init_output = match &res {
        Err(e) => {
            let error_output = serde_json::json!({
                "request": req,
                "error": e.to_string()
            });
            error_output
        }
        Ok(result) => {
            log::debug!(
                "completed search for request {}: {} edges in route, {} in tree",
                req,
                result.route.len(),
                result.tree.len()
            );

            let mut init_output = serde_json::json!({
                "request": req,
                "search_executed_time": result.search_start_time.to_rfc3339(),
                "search_runtime": result.search_runtime.hhmmss(),
                "total_runtime": result.total_runtime.hhmmss(),
                "route_edge_count": result.route.len(),
                "tree_edge_count": result.tree.len()
            });

            let route = result.route.to_vec();

            // build and append summaries if there is a route
            if let Some(et) = route.last() {
                // build instances of traversal and cost models to compute summaries
                let tmodel = match search_app.build_traversal_model(req) {
                    Err(e) => {
                        return vec![serde_json::json!({
                            "request": req,
                            "error": e.to_string()
                        })]
                    }
                    Ok(tmodel) => tmodel,
                };
                let cmodel =
                    match search_app.build_cost_model_for_traversal_model(req, tmodel.clone()) {
                        Err(e) => {
                            return vec![serde_json::json!({
                                "request": req,
                                "error": e.to_string()
                            })]
                        }
                        Ok(cmodel) => cmodel,
                    };

                let traversal_summary = tmodel.serialize_state_with_info(&et.result_state);
                let cost_summary = match cmodel.serialize_cost_with_info(&et.result_state) {
                    Err(e) => {
                        return vec![serde_json::json!({
                            "request": req,
                            "error": e.to_string()
                        })]
                    }
                    Ok(json) => json,
                };
                init_output["traversal_summary"] = traversal_summary;
                init_output["cost_summary"] = cost_summary;
            }

            init_output
        }
    };

    let init_acc: Result<Vec<serde_json::Value>, PluginError> = Ok(vec![init_output]);
    let json_result = output_plugins
        .iter()
        .fold(init_acc, |acc, p| {
            acc.and_then(|outer| {
                outer
                    .iter()
                    .map(|output| p.process(output, &res))
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
                "request": req,
                "error": e.to_string()
            })
        });
    match json_result {
        Err(e) => {
            vec![serde_json::json!({
                "request": req,
                "error": e.to_string()
            })]
        }
        Ok(json) => json,
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

        let app = CompassApp::try_from(conf_file_test.as_path())
            .or(CompassApp::try_from(conf_file_debug.as_path()))
            .unwrap();
        let query = serde_json::json!({
            "origin_vertex": 0,
            "destination_vertex": 2
        });
        let result = app.run(vec![query]).unwrap();
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
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
