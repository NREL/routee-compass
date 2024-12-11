use super::compass_app_configuration::CompassAppConfiguration;
use super::response::response_output_policy::ResponseOutputPolicy;
use super::response::response_sink::ResponseSink;
use super::{compass_app_ops as ops, config::compass_app_builder::CompassAppBuilder};
use crate::app::compass::response::response_persistence_policy::ResponsePersistencePolicy;
use crate::{
    app::{
        compass::{
            compass_app_error::CompassAppError,
            compass_input_field::CompassInputField,
            config::{
                compass_configuration_field::CompassConfigurationField,
                config_json_extension::ConfigJsonExtensions,
                cost_model::cost_model_builder::CostModelBuilder,
                graph_builder::DefaultGraphBuilder,
                termination_model_builder::TerminationModelBuilder,
            },
        },
        search::{search_app::SearchApp, search_app_result::SearchAppResult},
    },
    plugin::{
        input::{input_plugin::InputPlugin, input_plugin_ops as in_ops},
        output::{output_plugin::OutputPlugin, output_plugin_ops as out_ops},
    },
};
use chrono::{Duration, Local};
use config::Config;
use itertools::{Either, Itertools};
use kdam::{Bar, BarExt};
use rayon::{current_num_threads, prelude::*};
use routee_compass_core::algorithm::search::search_instance::SearchInstance;
use routee_compass_core::model::map::map_model::MapModel;
use routee_compass_core::model::map::map_model_config::MapModelConfig;
use routee_compass_core::model::state::state_model::StateModel;
use routee_compass_core::{
    algorithm::search::search_algorithm::SearchAlgorithm,
    util::duration_extension::DurationExtension,
};
use serde_json::Value;
use std::rc::Rc;
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
    pub search_app: Arc<SearchApp>,
    pub input_plugins: Vec<Arc<dyn InputPlugin>>,
    pub output_plugins: Vec<Arc<dyn OutputPlugin>>,
    pub configuration: CompassAppConfiguration,
}

impl CompassApp {
    /// Builds a CompassApp from a configuration TOML string, using a custom CompassAppBuilder.
    ///
    /// # Arguments
    ///
    /// * `config_string` - a string containing the configuration in TOML format
    /// * `original_file_path` - the original file path of the TOML
    /// * `builder` - a custom CompassAppBuilder instance
    ///
    /// # Returns
    ///
    /// * an instance of [`CompassApp`], or an error if load failed.
    pub fn try_from_config_toml_string(
        config_string: String,
        original_file_path: String,
        builder: &CompassAppBuilder,
    ) -> Result<Self, CompassAppError> {
        let config = ops::read_config_from_string(
            config_string.clone(),
            config::FileFormat::Toml,
            original_file_path,
        )?;
        let app = CompassApp::try_from((&config, builder))?;
        Ok(app)
    }
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

        let search_algorithm: SearchAlgorithm =
            config_json.get_config_serde(&CompassConfigurationField::Algorithm, &"TOML")?;

        let state_model = match config_json.get(CompassConfigurationField::State.to_string()) {
            Some(state_config) => Arc::new(StateModel::try_from(state_config)?),
            None => Arc::new(StateModel::empty()),
        };

        // build traversal model
        let traversal_start = Local::now();
        let traversal_params =
            config_json.get_config_section(CompassConfigurationField::Traversal, &"TOML")?;
        let traversal_model_service = builder.build_traversal_model_service(&traversal_params)?;
        let traversal_duration = (Local::now() - traversal_start)
            .to_std()
            .map_err(|e| CompassAppError::InternalError(e.to_string()))?;
        log::info!(
            "finished reading traversal model with duration {}",
            traversal_duration.hhmmss()
        );

        // build access model
        let access_start = Local::now();
        let access_params =
            config_json.get_config_section(CompassConfigurationField::Access, &"TOML")?;
        let access_model_service = builder.build_access_model_service(&access_params)?;
        let access_duration = (Local::now() - access_start)
            .to_std()
            .map_err(|e| CompassAppError::InternalError(e.to_string()))?;
        log::info!(
            "finished reading access model with duration {}",
            access_duration.hhmmss()
        );

        // build utility model
        let cost_params =
            config_json.get_config_section(CompassConfigurationField::Cost, &"TOML")?;
        let cost_model_service = CostModelBuilder {}.build(&cost_params)?;

        // build frontier model
        let frontier_start = Local::now();
        let frontier_params =
            config_json.get_config_section(CompassConfigurationField::Frontier, &"TOML")?;

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
            config_json.get_config_section(CompassConfigurationField::Termination, &"TOML")?;
        let termination_model = TerminationModelBuilder::build(&termination_model_json, None)?;

        // build graph
        let graph_start = Local::now();
        let graph_params =
            config_json.get_config_section(CompassConfigurationField::Graph, &"TOML")?;
        let graph = Arc::new(DefaultGraphBuilder::build(&graph_params)?);
        let graph_duration = (Local::now() - graph_start)
            .to_std()
            .map_err(|e| CompassAppError::InternalError(e.to_string()))?;
        log::info!(
            "finished reading graph with duration {}",
            graph_duration.hhmmss()
        );

        let graph_bytes = allocative::size_of_unique_allocated_data(&graph);
        log::info!("graph size: {} GB", graph_bytes as f64 / 1e9);

        let map_start = Local::now();
        let map_model_json = config_json.get(CompassConfigurationField::MapModel.to_str());
        let map_model_config =
            MapModelConfig::try_from(map_model_json).map_err(CompassAppError::BuildFailure)?;
        let map_model = Arc::new(MapModel::new(graph.clone(), map_model_config).map_err(|e| {
            CompassAppError::BuildFailure(format!("unable to load MapModel from config: {}", e))
        })?);
        let map_dur = to_std(Local::now() - map_start)?;
        log::info!(
            "finished loading map model with duration {}",
            map_dur.hhmmss()
        );

        #[cfg(debug_assertions)]
        {
            use std::io::Write;

            log::debug!("Building flamegraph for graph memory usage..");

            let mut flamegraph = allocative::FlameGraphBuilder::default();
            flamegraph.visit_root(&graph);
            let output = flamegraph.finish_and_write_flame_graph();

            let outdir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("target")
                .join("flamegraph");

            if !outdir.exists() {
                std::fs::create_dir(&outdir).unwrap();
            }

            let outfile = outdir.join("graph_memory_flamegraph.out");

            log::debug!("writing graph flamegraph to {:?}", outfile);

            let mut output_file = std::fs::File::create(outfile).unwrap();
            output_file.write_all(output.as_bytes()).unwrap();
        }

        // build search app
        let search_app = Arc::new(SearchApp::new(
            search_algorithm,
            graph,
            map_model,
            state_model,
            traversal_model_service,
            access_model_service,
            cost_model_service,
            frontier_model_service,
            termination_model,
        ));

        // build plugins
        let plugins_start = Local::now();
        let plugins_config =
            config_json.get_config_section(CompassConfigurationField::Plugins, &"TOML")?;

        let input_plugins = builder.build_input_plugins(&plugins_config)?;
        let output_plugins = builder.build_output_plugins(&plugins_config)?;

        let plugins_duration = to_std(Local::now() - plugins_start)?;
        log::info!(
            "finished loading plugins with duration {}",
            plugins_duration.hhmmss()
        );

        let configuration = CompassAppConfiguration::try_from(config)?;

        log::info!(
            "additional parameters - parallelism={}",
            configuration.parallelism,
        );

        Ok(CompassApp {
            search_app,
            input_plugins,
            output_plugins,
            configuration,
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
    ///
    /// # Arguments
    ///
    /// * `queries` - list of search queries to execute
    /// * `config` - configuration for this run batch which may override default configurations
    ///
    /// # Result
    ///
    /// if
    pub fn run(
        &self,
        queries: &mut [serde_json::Value],
        config: Option<&Value>,
    ) -> Result<Vec<Value>, CompassAppError> {
        // allow the user to overwrite global configurations for this run
        let parallelism: usize = get_optional_run_config(
            &CompassConfigurationField::Parallelism.to_str(),
            &"run configuration",
            config,
        )?
        .unwrap_or(self.configuration.parallelism);
        let response_persistence_policy: ResponsePersistencePolicy = get_optional_run_config(
            &CompassConfigurationField::ResponsePersistencePolicy.to_str(),
            &"run configuration",
            config,
        )?
        .unwrap_or(self.configuration.response_persistence_policy);
        let response_output_policy: ResponseOutputPolicy = get_optional_run_config(
            &CompassConfigurationField::ResponseOutputPolicy.to_str(),
            &"run configuration",
            config,
        )?
        .unwrap_or_else(|| self.configuration.response_output_policy.clone());
        let response_writer = response_output_policy.build()?;

        // INPUT PROCESSING
        let parallel_batch_size =
            (queries.len() as f64 / self.configuration.parallelism as f64).ceil() as usize;
        let input_plugin_result = apply_input_plugins(
            queries,
            &self.input_plugins,
            self.search_app.clone(),
            parallel_batch_size,
        )?;
        let (processed_inputs, input_errors) = input_plugin_result;
        let mut load_balanced_inputs =
            ops::apply_load_balancing_policy(processed_inputs, parallelism, 1.0)?;

        log::info!(
            "creating {} parallel batches across {} threads to run queries",
            self.configuration.parallelism,
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
            .map_err(|e| {
                CompassAppError::InternalError(format!("could not build progress bar: {}", e))
            })?;
        let search_pb_shared = Arc::new(Mutex::new(search_pb));

        // run parallel searches as organized by the (optional) load balancing policy
        // across a thread pool managed by rayon
        let run_query_result = match response_persistence_policy {
            ResponsePersistencePolicy::PersistResponseInMemory => run_batch_with_responses(
                &mut load_balanced_inputs,
                &self.output_plugins,
                &self.search_app,
                &response_writer,
                search_pb_shared,
            )?,
            ResponsePersistencePolicy::DiscardResponseFromMemory => run_batch_without_responses(
                &mut load_balanced_inputs,
                &self.output_plugins,
                &self.search_app,
                &response_writer,
                search_pb_shared,
            )?,
        };
        eprintln!();

        // combine successful runs along with any error rows for response
        let run_result = run_query_result
            // .chain(mapped_errors)
            .chain(input_errors)
            .collect();
        Ok(run_result)
    }
}

/// executes the input plugins on each query, returning all
/// successful mappings (left) and mapping errors (right) as the pair
/// (left, right). errors are already serialized into JSON.
fn apply_input_plugins(
    mapped_queries: &mut [Value],
    input_plugins: &Vec<Arc<dyn InputPlugin>>,
    search_app: Arc<SearchApp>,
    parallel_batch_size: usize,
) -> Result<(Vec<Value>, Vec<Value>), CompassAppError> {
    let input_pb = Bar::builder()
        .total(mapped_queries.len())
        .animation("fillup")
        .desc("input plugins")
        .build()
        .map_err(|e| {
            CompassAppError::InternalError(format!(
                "could not build input plugin progress bar: {}",
                e
            ))
        })?;
    let input_pb_shared = Arc::new(Mutex::new(input_pb));

    // input plugins need to be flattened, and queries that fail input processing need to be
    // returned at the end.
    let (good, bad): (Vec<_>, Vec<_>) = mapped_queries
        .par_chunks(parallel_batch_size)
        .map(|qs| {
            let (good, bad): (Vec<Vec<Value>>, Vec<Value>) = qs
                .iter()
                .map(|q| {
                    let mut plugin_state = serde_json::Value::Array(vec![q.to_owned()]);
                    for plugin in input_plugins {
                        let p = plugin.clone();
                        let op: in_ops::InputArrayOp =
                            Rc::new(|q| p.process(q, search_app.clone()));
                        in_ops::json_array_op(&mut plugin_state, op)?
                    }
                    let inner_processed = in_ops::json_array_flatten(&mut plugin_state)?;
                    // let inner_processed = apply_input_plugins(q, input_plugins);
                    if let Ok(mut pb_local) = input_pb_shared.lock() {
                        let _ = pb_local.update(1);
                    }
                    Ok(inner_processed)
                })
                .partition_map(|r| match r {
                    Ok(values) => Either::Left(values),
                    Err(error_response) => Either::Right(error_response),
                });

            (good.into_iter().flatten().collect_vec(), bad)
        })
        .unzip();
    eprintln!(); // end input plugin pb

    let result = (
        good.into_iter().flatten().collect_vec(),
        bad.into_iter().flatten().collect_vec(),
    );
    Ok(result)
}

pub fn get_optional_run_config<'a, K, T>(
    key: &K,
    parent_key: &K,
    config: Option<&serde_json::Value>,
) -> Result<Option<T>, CompassAppError>
where
    K: AsRef<str>,
    T: serde::de::DeserializeOwned + 'a,
{
    match config {
        Some(c) => {
            let value = c.get_config_serde_optional::<T>(key, parent_key)?;
            Ok(value)
        }
        None => Ok(None),
    }
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
    query: &mut serde_json::Value,
    output_plugins: &[Arc<dyn OutputPlugin>],
    search_app: &SearchApp,
) -> Result<serde_json::Value, CompassAppError> {
    let search_result = search_app.run(query);
    let output = apply_output_processing(query, search_result, search_app, output_plugins);
    Ok(output)
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

/// runs a query batch which has been sorted into parallel chunks
/// and retains the responses from each search in memory.
pub fn run_batch_with_responses(
    load_balanced_inputs: &mut Vec<Vec<Value>>,
    output_plugins: &[Arc<dyn OutputPlugin>],
    search_app: &SearchApp,
    response_writer: &ResponseSink,
    pb: Arc<Mutex<Bar>>,
) -> Result<Box<dyn Iterator<Item = Value>>, CompassAppError> {
    let run_query_result = load_balanced_inputs
        .par_iter_mut()
        .map(|queries| {
            queries
                .iter_mut()
                .map(|q| {
                    let mut response = run_single_query(q, output_plugins, search_app)?;
                    if let Ok(mut pb_local) = pb.lock() {
                        let _ = pb_local.update(1);
                    }
                    response_writer.write_response(&mut response)?;
                    Ok(response)
                })
                .collect::<Result<Vec<serde_json::Value>, CompassAppError>>()
        })
        .collect::<Result<Vec<Vec<serde_json::Value>>, CompassAppError>>()?;

    let run_result = run_query_result.into_iter().flatten();

    Ok(Box::new(run_result))
}

/// runs a query batch which has been sorted into parallel chunks.
/// the search result is not persisted in memory.
pub fn run_batch_without_responses(
    load_balanced_inputs: &mut Vec<Vec<Value>>,
    output_plugins: &[Arc<dyn OutputPlugin>],
    search_app: &SearchApp,
    response_writer: &ResponseSink,
    pb: Arc<Mutex<Bar>>,
) -> Result<Box<dyn Iterator<Item = Value>>, CompassAppError> {
    // run the computations, discard values that do not trigger an error
    let _ = load_balanced_inputs
        .par_iter_mut()
        .map(|queries| {
            // fold over query iterator allows us to propagate failures up while still using constant
            // memory to hold the state of the result object. we can't similarly return error values from
            // within a for loop or for_each call, and map creates more allocations. open to other ideas!
            let initial: Result<(), CompassAppError> = Ok(());
            let _ = queries.iter_mut().fold(initial, |_, q| {
                let mut response = run_single_query(q, output_plugins, search_app)?;
                if let Ok(mut pb_local) = pb.lock() {
                    let _ = pb_local.update(1);
                }
                response_writer.write_response(&mut response)?;
                Ok(())
            });
            Ok(())
        })
        .collect::<Result<Vec<_>, CompassAppError>>()?;

    Ok(Box::new(std::iter::empty::<Value>()))
}

// helper that applies the output processing. this includes
// 1. summarizing from the TraversalModel
// 2. applying the output plugins
pub fn apply_output_processing(
    request_json: &serde_json::Value,
    result: Result<(SearchAppResult, SearchInstance), CompassAppError>,
    search_app: &SearchApp,
    output_plugins: &[Arc<dyn OutputPlugin>],
) -> serde_json::Value {
    let mut initial: Value = match out_ops::create_initial_output(request_json, &result, search_app)
    {
        Ok(value) => value,
        Err(error_value) => return error_value,
    };
    for output_plugin in output_plugins.iter() {
        match output_plugin.process(&mut initial, &result) {
            Ok(()) => {}
            Err(e) => return out_ops::package_error(request_json, e),
        }
    }

    initial
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::app::compass::{
        compass_app_error::CompassAppError,
        config::compass_configuration_error::CompassConfigurationError,
    };

    use super::CompassApp;

    #[test]
    fn test_speeds() {
        let cwd_str = match std::env::current_dir() {
            Ok(cwd_path) => String::from(cwd_path.to_str().unwrap_or("<unknown>")),
            _ => String::from("<unknown>"),
        };
        // eprintln!("cwd           : {}", cwd_str);
        // eprintln!("Cargo.toml dir: {}", env!("CARGO_MANIFEST_DIR"));

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

        let app = match CompassApp::try_from(conf_file_test.as_path()) {
            Ok(a) => Ok(a),
            Err(CompassAppError::CompassConfigurationError(
                CompassConfigurationError::FileNormalizationNotFound(_key, _f1, _f2),
            )) => {
                // could just be the run location, depending on the environment/runner/IDE
                // try the alternative configuration that runs from the root directory
                CompassApp::try_from(conf_file_debug.as_path())
            }
            Err(other) => panic!("{}", other),
        }
        .unwrap();
        let query = serde_json::json!({
            "origin_vertex": 0,
            "destination_vertex": 2
        });
        let mut queries = vec![query];
        let result = app.run(&mut queries, None).unwrap();
        assert_eq!(result.len(), 1, "expected one result");
        // eprintln!("{}", serde_json::to_string_pretty(&result).unwrap());
        let route_0 = result[0].get("route").unwrap();
        let path_0 = route_0.get("path").unwrap();

        // path [1] is distance-optimal; path [0, 2] is time-optimal
        let expected_path = serde_json::json!(vec![0, 2]);
        assert_eq!(path_0, &expected_path);
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
