use std::sync::Arc;

use chrono::{Duration, Local};
use compass_core::{
    algorithm::search::min_search_tree::{
        a_star::{
            a_star::{backtrack_edges, run_a_star_edge_oriented},
            cost_estimate_function::{CostEstimateFunction, Haversine},
        },
        direction::Direction,
    },
    model::{
        graph::{directed_graph::DirectedGraph, edge_id::EdgeId},
        traversal::{
            free_flow_traversal_model::FreeFlowTraversalModel, state::search_state::SearchState,
            traversal_model::TraversalModel,
        },
    },
    util::read_only_lock::DriverReadOnlyLock,
};
use compass_tomtom::graph::{tomtom_graph::TomTomGraph, tomtom_graph_config::TomTomGraphConfig};
use log::{info, warn};
use rand::seq::SliceRandom;

fn main() {
    env_logger::init();
    let edges_path = "/Users/rfitzger/data/routee/tomtom/tomtom-condensed/edges_compass.csv.gz";
    let vertices_path =
        "/Users/rfitzger/data/routee/tomtom/tomtom-condensed/vertices_compass.csv.gz";
    let conf = TomTomGraphConfig {
        edge_list_csv: String::from(edges_path),
        vertex_list_csv: String::from(vertices_path),
        n_edges: Some(67198522),
        n_vertices: Some(56306871),
        verbose: true,
    };
    let graph = TomTomGraph::try_from(conf).unwrap();
    info!("{} rows in adjacency list", graph.adj.len());
    info!("{} rows in reverse list", graph.rev.len());
    info!("{} rows in edge list", graph.edges.len());
    info!("{} rows in vertex list", graph.vertices.len());
    info!("yay!");

    let haversine = Haversine {
        travel_speed_kph: 40.0,
    };
    let traversal_model = FreeFlowTraversalModel;

    let g = Arc::new(DriverReadOnlyLock::new(&graph as &dyn DirectedGraph));
    let h = Arc::new(DriverReadOnlyLock::new(
        &haversine as &dyn CostEstimateFunction,
    ));
    let t = Arc::new(DriverReadOnlyLock::new(
        &traversal_model as &dyn TraversalModel<State = SearchState>,
    ));

    let (o, d) = (
        graph.edges.choose(&mut rand::thread_rng()).unwrap().edge_id,
        graph.edges.choose(&mut rand::thread_rng()).unwrap().edge_id,
    );
    info!("randomly selected (origin, destination): ({}, {})", o, d);

    let g_e1 = Arc::new(g.read_only());
    let g_e2 = Arc::new(g.read_only());
    let h_e = Arc::new(h.read_only());
    let t_e = Arc::new(t.read_only());

    let start_time = Local::now();
    info!("running search");
    match run_a_star_edge_oriented(Direction::Forward, o, d, g_e1, t_e, h_e) {
        Err(e) => {
            info!("{}", e.to_string())
        }
        Ok(result) => {
            let duration = Local::now() - start_time;
            info!("finished search with duration {:?}", duration);
            info!("tree result has {} entries", result.len());
            log::logger().flush();
            if result.is_empty() {
                warn!("no path exists between requested origin and target")
            } else {
                let route = backtrack_edges(o, d, result, g_e2).unwrap();
                let g_erol3 = g.read_only();
                let g_e3 = g_erol3.read().unwrap();
                let src_vertex = g_e3.dst_vertex(o).unwrap();
                let dst_vertex = g_e3.src_vertex(d).unwrap();
                let src = g_e3.vertex_attr(src_vertex).unwrap().to_tuple_underlying();
                let dst = g_e3.vertex_attr(dst_vertex).unwrap().to_tuple_underlying();
                let time_ms = route
                    .clone()
                    .into_iter()
                    .map(|e| e.edge_cost())
                    .reduce(|x, y| x + y)
                    .unwrap();
                let dur = Duration::milliseconds(time_ms.0);
                let time_str = format!(
                    "{}:{}:{}",
                    dur.num_hours(),
                    dur.num_minutes() % 60,
                    dur.num_seconds() % 60
                );
                info!("origin, destination (x,y): {:?} {:?}", src, dst);
                info!("found route with {} edges", route.len());
                info!("route duration: {}", time_str);
                info!("done!");
            }
        }
    }
}
