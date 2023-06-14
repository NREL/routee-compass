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
    match TomTomGraph::try_from(conf) {
        Ok(graph) => {
            println!("{} rows in adjacency list", graph.adj.len());
            println!("{} rows in reverse list", graph.rev.len());
            println!("{} rows in edge list", graph.edges.len());
            println!("{} rows in vertex list", graph.vertices.len());
            println!("yay!")
        }
        Err(e) => {
            println!("uh oh, no good");
            println!("{}", e.to_string());
            panic!();
        }
    }
}
