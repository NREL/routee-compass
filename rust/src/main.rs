use rust_graph::{
    algorithm::dijkstra_shortest_path,
    graph::{Graph, Link, Node},
};

fn main() {
    let mut graph = Graph::new();

    let node1 = Node::new(1);
    let node2 = Node::new(2);
    let node3 = Node::new(3);
    let node4 = Node::new(4);

    // build a graph that has the shortest distance between node 1 and 4
    let link1 = Link {
        id: 1,
        start_node: node1,
        end_node: node2,
        road_class: 1,
        speed: 1,
        distance: 1,
        grade: 1,
        restriction: None,
    };
    let link2 = Link {
        id: 2,
        start_node: node2,
        end_node: node3,
        road_class: 1,
        speed: 1,
        distance: 1,
        grade: 1,
        restriction: None,
    };
    let link3 = Link {
        id: 3,
        start_node: node3,
        end_node: node4,
        road_class: 1,
        speed: 1,
        distance: 1,
        grade: 1,
        restriction: None,
    };
    let link4 = Link {
        id: 4,
        start_node: node1,
        end_node: node4,
        road_class: 1,
        speed: 1,
        distance: 1,
        grade: 1,
        restriction: None,
    };

    graph.add_edge(link1);
    graph.add_edge(link2);
    graph.add_edge(link3);
    graph.add_edge(link4);

    let result = dijkstra_shortest_path(&graph, &node1, &node4, |link| link.distance);

    println!("{:?}", result);
}
