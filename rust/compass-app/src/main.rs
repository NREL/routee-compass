use std::sync::Arc;

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
            free_flow_traversal_model::FreeFlowTraversalModel, traversal_model::TraversalModel,
        },
    },
    util::read_only_lock::DriverReadOnlyLock,
};
use compass_tomtom::graph::{tomtom_graph::TomTomGraph, tomtom_graph_config::TomTomGraphConfig};
use log::info;

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
    let empty_adj_rows = graph
        .adj
        .to_owned()
        .into_iter()
        .map(|map| map.values().len())
        .fold(vec![0; 10], |mut acc, cnt| {
            acc[cnt] = acc[cnt] + 1;
            acc
        });
    info!("{} rows in adjacency list", graph.adj.len());
    info!(
        "{:?} adj histogram vertices by out link counts",
        empty_adj_rows
    );
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
        &traversal_model as &dyn TraversalModel<State = i64>,
    ));

    let (o, d) = (EdgeId(123), EdgeId(456));

    let g_e1 = Arc::new(g.read_only());
    let g_e2 = Arc::new(g.read_only());
    let h_e = Arc::new(h.read_only());
    let t_e = Arc::new(t.read_only());

    info!("running search");
    match run_a_star_edge_oriented(Direction::Forward, o, d, g_e1, t_e, h_e) {
        Err(e) => {
            info!("{}", e.to_string())
        }
        Ok(result) => {
            info!("tree result has {} entries", result.len());
            log::logger().flush();
            let route = backtrack_edges(o, d, result, g_e2).unwrap();
            let mut route_edge_ids: Vec<EdgeId> = route.iter().map(|r| r.edge_id).collect();
            route_edge_ids.insert(0, o);
            route_edge_ids.push(d);
            info!("{:?}", route_edge_ids)
        }
    }
}
