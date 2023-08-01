use chrono::Local;
use compass_app::app::app_error::AppError;
use compass_app::app::search::search_app::SearchApp;
use compass_app::config::app_config::AppConfig;
use compass_app::config::graph::GraphConfig;
use compass_core::algorithm::search::min_search_tree::a_star::cost_estimate_function::Haversine;
use compass_core::model::traversal::traversal_model::TraversalModel;
use compass_core::model::units::Velocity;
use compass_tomtom::graph::{tomtom_graph::TomTomGraph, tomtom_graph_config::TomTomGraphConfig};
use log;
use rand::seq::SliceRandom;
use std::env;
use std::error::Error;
use std::path::PathBuf;
use uom::si::velocity::kilometer_per_hour;

fn main() -> Result<(), Box<dyn Error>> {
    let setup_start = Local::now();
    env_logger::init();
    let args = env::args().collect::<Vec<String>>();
    let config_file_string = match args.get(1) {
        None => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "missing CLI argument 1 for compass-app configuration TOML file",
        )),
        Some(arg) => Ok(arg),
    }?;

    let config = AppConfig::from_path(PathBuf::from(config_file_string))?;

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

    let setup_duration = Local::now() - setup_start;
    log::info!("finished setup with duration {:?}", setup_duration);

    let search_start = Local::now();
    log::info!("running search");
    let results = compass_app.run_edge_oriented(queries)?;
    let search_duration = Local::now() - search_start;
    log::info!("finished search with duration {:?}", search_duration);

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
