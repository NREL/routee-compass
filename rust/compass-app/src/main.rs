use chrono::Local;
use clap::Parser;
use compass_app::app::app_error::AppError;
use compass_app::app::search::search_app::SearchApp;
use compass_app::cli::CLIArgs;
use compass_app::config::app_config::AppConfig;
use compass_app::config::graph::GraphConfig;
use compass_app::plugin::input::InputPlugin;
use compass_app::plugin::output::OutputPlugin;
use compass_app::plugin::plugin_error::PluginError;
use compass_core::model::cost::cost::Cost;
use compass_core::model::traversal::traversal_model::TraversalModel;
use compass_core::model::units::{TimeUnit, Velocity};
use compass_core::util::duration_extension::DurationExtension;
use compass_core::{
    algorithm::search::min_search_tree::a_star::cost_estimate_function::Haversine,
    model::graph::edge_id::EdgeId,
};
use compass_tomtom::graph::{tomtom_graph::TomTomGraph, tomtom_graph_config::TomTomGraphConfig};
use log::info;
use rand::seq::SliceRandom;
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

    // read query json file into a serde json Value
    let query_file = File::open(args.query_file)?;
    let n_queries = 10;

    let reader = BufReader::new(query_file);
    let query: serde_json::Value = serde_json::from_reader(reader)?;

    info!("Query: {:?}", query);

    let graph_start = Local::now();
    let graph = match config.graph {
        GraphConfig::TomTom {
            edge_file,
            vertex_file,
            n_edges,
            n_vertices,
            verbose,
        } => {
            let conf = TomTomGraphConfig {
                edge_list_csv: edge_file,
                vertex_list_csv: vertex_file,
                n_edges,
                n_vertices,
                verbose,
            };
            let graph = TomTomGraph::try_from(conf)?;
            graph
        }
    };
    let graph_duration = (Local::now() - graph_start).to_std()?;
    log::info!(
        "finished reading graph with duration {}",
        graph_duration.hhmmss()
    );

    let haversine = Haversine {
        travel_speed: Velocity::new::<kilometer_per_hour>(40.0),
        output_unit: TimeUnit::Milliseconds,
    };

    let traversal_start = Local::now();
    let traversal_model: TraversalModel = config.search.traversal_model.try_into()?;
    let traversal_duration = (Local::now() - traversal_start).to_std()?;
    log::info!(
        "finished reading traversal model with duration {}",
        traversal_duration.hhmmss()
    );

    let search_app: SearchApp = SearchApp::new(&graph, &traversal_model, &haversine, Some(2));

    let input_plugins: Vec<InputPlugin> = config
        .plugin
        .input_plugins
        .iter()
        .map(InputPlugin::try_from)
        .collect::<Result<Vec<InputPlugin>, PluginError>>()?;

    let output_plugins: Vec<OutputPlugin> = config
        .plugin
        .output_plugins
        .iter()
        .map(OutputPlugin::try_from)
        .collect::<Result<Vec<OutputPlugin>, PluginError>>()?;

    let queries_result: Result<Vec<(EdgeId, EdgeId)>, AppError> = (0..n_queries)
        .map(|_| {
            let (o, d) = (
                graph
                    .edges
                    .choose(&mut rand::thread_rng())
                    .ok_or(AppError::InternalError(String::from(
                        "graph.edges.choose returned None",
                    )))?
                    .edge_id,
                graph
                    .edges
                    .choose(&mut rand::thread_rng())
                    .ok_or(AppError::InternalError(String::from(
                        "graph.edges.choose returned None",
                    )))?
                    .edge_id,
            );
            log::info!("randomly selected (origin, destination): ({}, {})", o, d);
            Ok((o, d))
        })
        .collect();

    // in the future, "queries" should be parsed from the user at the top of the app
    let queries = queries_result?;

    let search_start = Local::now();
    log::info!("running search");
    let results = search_app.run_edge_oriented(queries.clone())?;
    let search_duration = (Local::now() - search_start).to_std()?;
    log::info!("finished search with duration {}", search_duration.hhmmss());

    let output_start = Local::now();
    let output_rows = queries
        .clone()
        .iter()
        .zip(results)
        .map(move |((o, d), r)| match r {
            Err(e) => {
                let error_output = serde_json::json!({
                    "origin_edge_id": o,
                    "destination_edge_id": d,
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
                    "origin_edge_id": result.origin,
                    "destination_edge_id": result.destination,
                    // "duration": dur
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
                            "origin_edge_id": o,
                            "destination_edge_id": d,
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
