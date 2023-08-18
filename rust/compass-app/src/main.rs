use chrono::Local;
use clap::Parser;
use compass_app::app::app_error::AppError;
use compass_app::app::compass::compass_json_extensions::CompassJsonExtensions;
use compass_app::app::search::search_app::SearchApp;
use compass_app::cli::CLIArgs;
use compass_app::config::app_config::AppConfig;
use compass_app::config::graph::GraphConfig;
use compass_app::plugin::input::{input_plugin_ops, InputPlugin};
use compass_app::plugin::output::OutputPlugin;
use compass_app::plugin::plugin_error::PluginError;
use compass_core::algorithm::search::min_search_tree::a_star::cost_estimate_function::Haversine;
use compass_core::model::cost::cost::Cost;
use compass_core::model::traversal::traversal_model::TraversalModel;
use compass_core::model::units::{TimeUnit, Velocity};
use compass_core::util::duration_extension::DurationExtension;
use compass_tomtom::graph::{tomtom_graph::TomTomGraph, tomtom_graph_config::TomTomGraphConfig};
use log::info;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use uom::si::velocity::kilometer_per_hour;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let args = CLIArgs::parse();

    let config = match args.config {
        Some(config_file) => {
            let config = AppConfig::from_path(&config_file)?;
            info!("Using config file: {:?}", config_file);
            config
        }
        None => {
            let config = AppConfig::default()?;
            info!("Using default config");
            config
        }
    };
    info!("Config: {:?}", config);

    let search_app_start = Local::now();
    let search_app: SearchApp = SearchApp::try_from(&config)?;
    let search_app_duration = (Local::now() - search_app_start).to_std()?;
    log::info!(
        "finished building search app with duration {}",
        search_app_duration.hhmmss()
    );

    let plugins_start = Local::now();
    let input_plugins: Vec<InputPlugin> = config
        .plugin
        .input_plugins
        .iter()
        .map(InputPlugin::try_from)
        .collect::<Result<Vec<InputPlugin>, PluginError>>()?;

    let output_plugins: Vec<OutputPlugin> = &config
        .plugin
        .output_plugins
        .iter()
        .map(OutputPlugin::try_from)
        .collect::<Result<Vec<OutputPlugin>, PluginError>>()?;
    let plugins_duration = (Local::now() - plugins_start).to_std()?;
    log::info!(
        "finished loading plugins with duration {}",
        plugins_duration.hhmmss()
    );

    let query_file = File::open(args.query_file)?;
    let reader = BufReader::new(query_file);
    let user_json: serde_json::Value =
        serde_json::from_reader(reader).map_err(AppError::CodecError)?;
    let user_queries = user_json.get_queries()?;
    info!("Query: {:?}", user_json);

    let processed_user_queries =
        input_plugin_ops::apply_input_plugins(user_queries, input_plugins)?;

    let search_start = Local::now();
    log::info!("running search");
    let results = search_app.run_vertex_oriented(processed_user_queries.clone())?;
    let search_duration = (Local::now() - search_start).to_std()?;
    log::info!("finished search with duration {}", search_duration.hhmmss());

    let output_start = Local::now();
    let output_rows = processed_user_queries
        .clone()
        .iter()
        .zip(results)
        .map(move |(req, res)| match res {
            Err(e) => {
                let error_output = serde_json::json!({
                    "request": req,
                    "error": e.to_string()
                });
                // log::error!("({},{}) failed: {}", o, d, e);
                error_output
            }
            Ok(result) => {
                let links: usize = result.route.clone().len();
                let mut time_millis = Cost::ZERO;
                for traversal in result.route.clone() {
                    let cost = traversal.edge_cost();
                    time_millis = time_millis + cost;
                }
                // whether time cost is ms actually depends on user settings, though safe bet for now
                // let dur = Duration::from_millis((time_millis.0).0 as u64).hhmmss();
                log::info!(
                    "({}) -> ({}) had route with {} links, tree with {} links",
                    result.origin,
                    result.destination,
                    links,
                    result.tree_size,
                    // dur
                );

                let init_output = serde_json::json!({
                    "request": req,
                    "search_runtime": result.search_runtime.hhmmss(),
                    "route_runtime": result.route_runtime.hhmmss(),
                    "tree_size": result.tree_size
                });
                let route = result.route.to_vec();
                let init_acc: Result<serde_json::Value, PluginError> = Ok(init_output);
                let json_result = output_plugins
                    .iter()
                    .fold(init_acc, move |acc, plugin| match acc {
                        Err(e) => Err(e),
                        Ok(json) => plugin(&json, Ok(&route)),
                    })
                    .map_err(AppError::PluginError);
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
        })
        .collect::<Vec<serde_json::Value>>();
    let output_contents = serde_json::to_string(&output_rows)?;
    std::fs::write("result.json", output_contents)?;

    let output_duration = (Local::now() - output_start).to_std()?;
    log::info!(
        "finished generating output with duration {}",
        output_duration.hhmmss()
    );
    return Ok(());
}
