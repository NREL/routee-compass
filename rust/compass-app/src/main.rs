use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use chrono::Local;
use clap::Parser;

use compass_app::app::app_error::AppError;
use compass_app::app::search::search_app::SearchApp;
use compass_app::cli::CLIArgs;
use compass_app::config::app_config::AppConfig;
use compass_app::config::graph::GraphConfig;
use compass_core::algorithm::search::min_search_tree::a_star::cost_estimate_function::Haversine;
use compass_core::model::traversal::traversal_model::TraversalModel;
use compass_core::model::units::Velocity;
use compass_tomtom::graph::{tomtom_graph::TomTomGraph, tomtom_graph_config::TomTomGraphConfig};
use rand::seq::SliceRandom;
use uom::si::velocity::kilometer_per_hour;

use log::info;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let args = CLIArgs::parse();

    let config = match args.config {
        Some(config_file) => {
            let config = AppConfig::from_path(&config_file).unwrap();
            info!("Using config file: {:?}", config_file);
            config
        }
        None => {
            let config = AppConfig::default().unwrap();
            info!("Using default config");
            config
        }
    };

    // read query json file into a serde json Value
    let query_file = File::open(args.query_file).unwrap();
    info!("Using query file: {:?}", query_file);

    let reader = BufReader::new(query_file);
    let query: serde_json::Value = serde_json::from_reader(reader).unwrap();

    info!("Query: {:?}", query);

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
            log::info!("{} rows in adjacency list", graph.adj.len());
            log::info!("{} rows in reverse list", graph.rev.len());
            log::info!("{} rows in edge list", graph.edges.len());
            log::info!("{} rows in vertex list", graph.vertices.len());
            log::info!("yay!");
            graph
        }
    };

    let haversine = Haversine {
        travel_speed: Velocity::new::<kilometer_per_hour>(40.0),
    };
    let traversal_model: TraversalModel = config.search.traversal_model.try_into()?;
    let compass_app: SearchApp = SearchApp::new(&graph, &traversal_model, &haversine);

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

    // in the future, "queries" should be parsed from the user at the top of the app
    let queries = vec![(o, d)];

    let start_time = Local::now();
    log::info!("running search");
    let results = compass_app.run_edge_oriented(queries)?;
    let duration = Local::now() - start_time;
    log::info!("finished search with duration {:?}", duration);

    // write each search result to it's own CSV file in the current working directory
    for result in results {
        let out = PathBuf::from(format!("{}-{}.csv", result.origin, result.destination));
        let mut writer = csv::Writer::from_path(out)?;
        for traversal in result.route {
            writer.serialize(traversal)?;
        }
    }
    return Ok(());
}
