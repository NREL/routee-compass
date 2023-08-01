use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::time::Duration;

use chrono::Local;
use clap::Parser;
use log::info;
use rand::seq::SliceRandom;
use uom::si::velocity::kilometer_per_hour;

use compass_app::app::app_error::AppError;
use compass_app::app::search::search_app::SearchApp;
use compass_app::cli::CLIArgs;
use compass_app::config::app_config::AppConfig;
use compass_app::config::graph::GraphConfig;
use compass_core::algorithm::search::min_search_tree::a_star::cost_estimate_function::Haversine;
use compass_core::model::cost::cost::Cost;
use compass_core::model::traversal::traversal_model::TraversalModel;
use compass_core::model::units::Velocity;
use compass_tomtom::graph::{tomtom_graph::TomTomGraph, tomtom_graph_config::TomTomGraphConfig};

fn main() -> Result<(), Box<dyn Error>> {
    let setup_start = Local::now();
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

    let reader = BufReader::new(query_file);
    let query: serde_json::Value = serde_json::from_reader(reader)?;

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
    let search_app: SearchApp = SearchApp::new(&graph, &traversal_model, &haversine);

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

    let setup_duration = Local::now() - setup_start;
    log::info!("finished setup with duration {:?}", setup_duration);

    let search_start = Local::now();
    log::info!("running search");
    let results = search_app.run_edge_oriented(queries)?;
    let search_duration = Local::now() - search_start;
    log::info!("finished search with duration {:?}", search_duration);

    // (replace this section with output plugins)
    for result in results {
        let links = result.route.clone().len();
        let mut time_secs = Cost::ZERO;
        for traversal in result.route {
            let cost = traversal.edge_cost();
            time_secs = time_secs + cost;
        }
        let dur = Duration::from_secs_f64((time_secs.0).0);
        log::info!(
            "{} -> {} had {} links, total time of {:?}",
            result.origin,
            result.destination,
            links,
            dur
        );
    }
    return Ok(());
}
