use std::sync::Arc;

use compass_core::{
    algorithm::search::min_search_tree::{
        a_star::{
            a_star::{backtrack, run_a_star},
            cost_estimate_function::{CostEstimateFunction, Haversine},
        },
        direction::Direction,
    },
    model::{
        graph::{directed_graph::DirectedGraph, edge_id::EdgeId, vertex_id::VertexId},
        traversal::{
            free_flow_traversal_model::FreeFlowTraversalModel, traversal_model::TraversalModel,
        },
    },
    util::read_only_lock::DriverReadOnlyLock,
};
use compass_tomtom::graph::{tomtom_graph::TomTomGraph, tomtom_graph_config::TomTomGraphConfig};
// use kdam::tqdm;

fn main() {
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

    let g_e = Arc::new(g.read_only());
    let h_e = Arc::new(h.read_only());
    let t_e = Arc::new(t.read_only());

    let (o, d) = (VertexId(123), VertexId(456));

    match run_a_star(Direction::Forward, o, d, g_e, t_e, h_e) {
        Err(e) => {
            println!("{}", e.to_string())
        }
        Ok(result) => {
            let route = backtrack(o, d, result).unwrap();
            let route_edges: Vec<EdgeId> = route.iter().map(|r| r.edge_id).collect();
            println!("{:?}", route_edges)
        }
    }

    // match TomTomGraph::try_from(conf) {
    //     Ok(graph) => {
    //         println!("{} rows in adjacency list", graph.adj.len());
    //         println!("{} rows in reverse list", graph.rev.len());
    //         println!("{} rows in edge list", graph.edges.len());
    //         println!("{} rows in vertex list", graph.vertices.len());
    //         println!("yay!")
    //     }
    //     Err(e) => {
    //         println!("uh oh, no good");
    //         println!("{}", e.to_string());
    //         panic!();
    //     }
    // }
}
