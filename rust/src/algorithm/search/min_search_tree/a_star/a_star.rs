use keyed_priority_queue::KeyedPriorityQueue;
use std::collections::HashMap;
use std::sync::RwLockReadGuard;

use super::a_star_frontier::AStarFrontier;
use super::a_star_traversal::AStarTraversal;
use super::cost_estimate_function::CostEstimateFunction;
use crate::algorithm::search::edge_traversal::EdgeTraversal;
use crate::algorithm::search::search_error::SearchError;
use crate::model::cost::cost::Cost;
use crate::model::graph::edge_id::EdgeId;
use crate::model::traversal::traversal_model::TraversalModel;
use crate::util::read_only_lock::ExecutorReadOnlyLock;
use crate::{
    algorithm::search::min_search_tree::direction::Direction,
    model::graph::{directed_graph::DirectedGraph, vertex_id::VertexId},
};
use std::sync::Arc;

type MinSearchTree<S> = HashMap<VertexId, AStarTraversal<S>>;

///
/// run an A* Search over the given directed graph model. traverses links
/// from the source, via the provided direction, to the target. uses the
/// provided traversal model for state updates and link costs. estimates
/// the distance to the destination (the a* heuristic) using the provided
/// cost estimate function.
pub fn run_a_star<S: Sync + Send + Eq + Copy + Clone>(
    direction: Direction,
    source: VertexId,
    target: VertexId,
    directed_graph: Arc<ExecutorReadOnlyLock<&dyn DirectedGraph>>,
    traversal_model: Arc<ExecutorReadOnlyLock<&dyn TraversalModel<State = S>>>,
    cost_estimate_fn: Arc<ExecutorReadOnlyLock<&dyn CostEstimateFunction>>,
) -> Result<MinSearchTree<S>, SearchError> {
    // context for the search (graph, search functions, frontier priority queue)
    let g = directed_graph.read().unwrap();
    let m = traversal_model.read().unwrap();
    let c = cost_estimate_fn.read().unwrap();
    let mut open_set: KeyedPriorityQueue<AStarFrontier<S>, Cost> = KeyedPriorityQueue::new();
    let mut g_score: HashMap<VertexId, Cost> = HashMap::new();
    let mut f_score: HashMap<VertexId, Cost> = HashMap::new();
    let mut solution: HashMap<VertexId, AStarTraversal<S>> = HashMap::new();

    // setup initial search state
    g_score.insert(source, Cost::ZERO);
    f_score.insert(source, h_cost(source, target, &c, &g)?);
    let origin = AStarFrontier {
        vertex_id: source,
        prev_edge_id: None,
        state: m.initial_state()?,
    };
    open_set.push(origin, Cost::ZERO);

    // run search
    loop {
        match open_set.pop() {
            None => break,
            Some((current, _)) if current.vertex_id == target => break,
            Some((current, _)) => {
                let triplets = g
                    .incident_triplets(current.vertex_id, direction)
                    .map_err(SearchError::GraphCorrectnessFailure)?;

                for (src_id, edge_id, dst_id) in triplets {
                    let et =
                        EdgeTraversal::new(edge_id, current.prev_edge_id, current.state, &g, &m)?;
                    let h_cost_value = h_cost(src_id, target, &c, &g)?;
                    let src_gscore = g_score.get(&src_id).unwrap_or(&Cost::INFINITY);
                    let tentative_gscore = *src_gscore + et.access_cost + et.traversal_cost;
                    let dst_gscore = g_score.get(&dst_id).unwrap_or(&Cost::INFINITY);
                    if tentative_gscore < *dst_gscore {
                        let f_score_value = tentative_gscore + h_cost_value;
                        let traversal = AStarTraversal {
                            terminal_vertex: dst_id,
                            edge_traversal: et,
                        };
                        g_score.insert(dst_id, tentative_gscore);
                        f_score.insert(dst_id, f_score_value);
                        solution.insert(dst_id, traversal);

                        let f = AStarFrontier {
                            vertex_id: dst_id,
                            prev_edge_id: Some(edge_id),
                            state: et.result_state,
                        };
                        match open_set.get_priority(&f) {
                            None => {
                                open_set.push(f, f_score_value);
                            }
                            Some(_) => {}
                        }
                    }
                }
            }
        }
    }

    return Ok(solution);
}

pub fn run_a_star_edge_oriented<S: Sync + Send + Eq + Copy + Clone>(
    direction: Direction,
    source: EdgeId,
    target: EdgeId,
    directed_graph: Arc<ExecutorReadOnlyLock<&dyn DirectedGraph>>,
    traversal_model: Arc<ExecutorReadOnlyLock<&dyn TraversalModel<State = S>>>,
    cost_estimate_fn: Arc<ExecutorReadOnlyLock<&dyn CostEstimateFunction>>,
) -> Result<MinSearchTree<S>, SearchError> {
    // 1. guard against edge conditions (src==dst, src.dst_v == dst.src_v)
    // 2. get destination vertex of source edge, source vertex of destination edge
    // 3. run a star for those vertices
    // 3. prepend source edge, append destination edge to min search tree (with no costs added, for now)
    todo!()
}

pub fn backtrack<S: Copy + Clone>(
    source_id: VertexId,
    target_id: VertexId,
    solution: HashMap<VertexId, AStarTraversal<S>>,
) -> Result<Vec<EdgeTraversal<S>>, SearchError> {
    let mut result: Vec<EdgeTraversal<S>> = vec![];
    let mut this_vertex = target_id.clone();
    loop {
        if this_vertex == source_id {
            break;
        }
        let traversal = solution
            .get(&this_vertex)
            .ok_or(SearchError::VertexMissingFromSearchTree(this_vertex))?;
        result.push(traversal.edge_traversal.clone());
        this_vertex = traversal.terminal_vertex;
    }
    Ok(result)
}

fn h_cost(
    vertex_id: VertexId,
    target_id: VertexId,
    c: &RwLockReadGuard<&dyn CostEstimateFunction>,
    g: &RwLockReadGuard<&dyn DirectedGraph>,
) -> Result<Cost, SearchError> {
    let src_v = g
        .vertex_attr(vertex_id)
        .map_err(SearchError::GraphCorrectnessFailure)?;
    let dst_v = g
        .vertex_attr(target_id)
        .map_err(SearchError::GraphCorrectnessFailure)?;
    c.cost(src_v, dst_v)
        .map_err(SearchError::CostCalculationError)
}

#[cfg(test)]
mod tests {
    use crate::{
        algorithm::search::min_search_tree::dijkstra::edge_frontier::EdgeFrontier,
        model::{
            cost::cost_error::CostError,
            graph::{edge_id::EdgeId, graph_error::GraphError},
            property::{edge::Edge, road_class::RoadClass, vertex::Vertex},
            traversal::traversal_error::TraversalError,
            units::{
                centimeters::Centimeters, cm_per_second::CmPerSecond, millis::Millis,
                ordinate::Ordinate,
            },
        },
        util::read_only_lock::{DriverReadOnlyLock, ExecutorReadOnlyLock},
    };
    use rayon::prelude::*;

    use super::*;

    struct TestModel;
    impl TraversalModel for TestModel {
        type State = i64;
        fn initial_state(&self) -> Result<Self::State, TraversalError> {
            Ok(0)
        }

        fn traversal_cost(
            &self,
            e: &Edge,
            state: &Self::State,
        ) -> Result<(Cost, Self::State), TraversalError> {
            let c = *state as f64
                + (e.distance_centimeters.0 as f64 / e.free_flow_speed_seconds.0 as f64);
            let c64 = c as i64;
            Ok((Cost(c64), c64))
        }

        fn access_cost(
            &self,
            src: &Edge,
            dst: &Edge,
            state: &Self::State,
        ) -> Result<(Cost, Self::State), TraversalError> {
            Ok((Cost::ZERO, state.clone()))
        }

        fn valid_frontier(
            &self,
            frontier: &EdgeFrontier<Self::State>,
        ) -> Result<bool, TraversalError> {
            Ok(true)
        }

        fn terminate_search(
            &self,
            frontier: &EdgeFrontier<Self::State>,
        ) -> Result<bool, TraversalError> {
            Ok(false)
        }
    }

    struct TestDG<'a> {
        adj: &'a HashMap<VertexId, HashMap<EdgeId, VertexId>>,
        edges: HashMap<EdgeId, Edge>,
    }
    impl DirectedGraph for TestDG<'_> {
        fn edge_attr(&self, edge_id: EdgeId) -> Result<Edge, GraphError> {
            match self.edges.get(&edge_id) {
                None => Err(GraphError::EdgeAttributeNotFound { edge_id }),
                Some(edge) => Ok(*edge),
            }
        }
        fn vertex_attr(&self, _vertex_id: VertexId) -> Result<Vertex, GraphError> {
            Ok(Vertex {
                x: Ordinate(0),
                y: Ordinate(0),
            })
        }
        fn out_edges(&self, src: VertexId) -> Result<Vec<EdgeId>, GraphError> {
            match self.adj.get(&src) {
                None => Err(GraphError::VertexWithoutOutEdges { vertex_id: src }),
                Some(out_map) => Ok(out_map.keys().cloned().collect()),
            }
        }
        fn in_edges(&self, _src: VertexId) -> Result<Vec<EdgeId>, GraphError> {
            Err(GraphError::TestError)
            // match self.adj.values()..get(&src) {
            //     None => Err(GraphError::VertexWithoutInEdges { vertex_id: src }),
            //     Some(out_map) => Ok(out_map.keys().cloned().collect()),
            // }
        }
        fn src_vertex(&self, edge_id: EdgeId) -> Result<VertexId, GraphError> {
            self.edge_attr(edge_id).map(|e| e.start_vertex)
        }
        fn dst_vertex(&self, edge_id: EdgeId) -> Result<VertexId, GraphError> {
            self.edge_attr(edge_id).map(|e: Edge| e.end_vertex)
        }
    }

    impl<'a> TestDG<'a> {
        fn new(
            adj: &'a HashMap<VertexId, HashMap<EdgeId, VertexId>>,
            edges_cps: HashMap<EdgeId, CmPerSecond>,
        ) -> Result<TestDG<'a>, GraphError> {
            let mut edges: HashMap<EdgeId, Edge> = HashMap::new();
            for (src, out_edges) in adj {
                for (edge_id, dst) in out_edges {
                    let cps = edges_cps
                        .get(&edge_id)
                        .ok_or(GraphError::EdgeIdNotFound { edge_id: *edge_id })?;
                    let edge = Edge {
                        start_vertex: *src,
                        end_vertex: *dst,
                        road_class: RoadClass(0),
                        free_flow_speed_seconds: cps.clone(),
                        distance_centimeters: Centimeters(100),
                        grade_millis: Millis(0),
                    };
                    edges.insert(*edge_id, edge);
                }
            }

            Ok(TestDG { adj, edges })
        }
    }

    struct TestCost;
    impl CostEstimateFunction for TestCost {
        fn cost(&self, _src: Vertex, _dst: Vertex) -> Result<Cost, CostError> {
            Ok(Cost(5))
        }
    }

    #[test]
    fn test_e2e_queries() {
        // these mocks stand-in for the trait objects required by the search function
        let driver_cf_obj = TestCost;
        let driver_cf = Arc::new(DriverReadOnlyLock::new(
            &driver_cf_obj as &dyn CostEstimateFunction,
        ));

        // simple box world but no one should drive between (0) and (1) because of slow speeds
        // (0) <---> (1)
        //  ^         ^
        //  |         |
        //  v         v
        // (3) <---> (2)
        // (0) -[0]-> (1) slow
        // (1) -[1]-> (0) slow
        // (1) -[2]-> (2) med
        // (2) -[3]-> (1) med
        // (2) -[4]-> (3) med
        // (3) -[5]-> (2) med
        // (3) -[6]-> (0) fast
        // (0) -[7]-> (3) fast
        let adj = HashMap::from([
            (VertexId(0), HashMap::from([(EdgeId(0), VertexId(1))])),
            (VertexId(1), HashMap::from([(EdgeId(1), VertexId(0))])),
            (VertexId(1), HashMap::from([(EdgeId(2), VertexId(2))])),
            (VertexId(2), HashMap::from([(EdgeId(3), VertexId(1))])),
            (VertexId(2), HashMap::from([(EdgeId(4), VertexId(3))])),
            (VertexId(3), HashMap::from([(EdgeId(5), VertexId(2))])),
            (VertexId(3), HashMap::from([(EdgeId(6), VertexId(0))])),
            (VertexId(0), HashMap::from([(EdgeId(7), VertexId(3))])),
        ]);
        let edges_cps = HashMap::from([
            (EdgeId(0), CmPerSecond(10)),  // 10 seconds
            (EdgeId(1), CmPerSecond(10)),  // 10 seconds
            (EdgeId(2), CmPerSecond(50)),  // 2 seconds
            (EdgeId(3), CmPerSecond(50)),  // 2 seconds
            (EdgeId(4), CmPerSecond(50)),  // 2 seconds
            (EdgeId(5), CmPerSecond(50)),  // 2 seconds
            (EdgeId(6), CmPerSecond(100)), // 1 second
            (EdgeId(7), CmPerSecond(100)), // 1 second
        ]);
        let driver_dg_obj = TestDG::new(&adj, edges_cps).unwrap();
        let driver_dg = Arc::new(DriverReadOnlyLock::new(
            &driver_dg_obj as &dyn DirectedGraph,
        ));
        let driver_tm_obj = TestModel;
        let driver_tm = Arc::new(DriverReadOnlyLock::new(
            &driver_tm_obj as &dyn TraversalModel<State = i64>,
        ));

        // todo:
        // - setup the road network to play well with the test queries (grid world)
        // - confirm that we can parallelize queries with shared memory
        // - handle result of fork with a join and test of Result<>

        let queries: Vec<(VertexId, VertexId)> = vec![
            (VertexId(0), VertexId(1)), // 0 -> 3 -> 2 -> 1
            (VertexId(0), VertexId(3)), // 0 -> 3
            (VertexId(1), VertexId(0)), // 1 -> 2 -> 3 -> 0
            (VertexId(1), VertexId(2)), // 1 -> 2
            (VertexId(2), VertexId(3)), // 2 -> 3
        ];

        let result: Vec<Result<MinSearchTree<i64>, SearchError>> = queries
            .clone()
            .into_par_iter()
            .map(|(o, d)| {
                let dg_inner = Arc::new(driver_dg.read_only());
                let tm_inner = Arc::new(driver_tm.read_only());
                let cost_inner = Arc::new(driver_cf.read_only());
                run_a_star(Direction::Forward, o, d, dg_inner, tm_inner, cost_inner)
            })
            .collect();

        for (r, q) in result.into_iter().zip(queries) {
            let msg = match r {
                Err(e) => e.to_string(),
                Ok(solution) => {
                    let query = format!("({} -> {})", q.0.to_string(), q.1.to_string());
                    let length = solution.len();
                    let route = backtrack(q.0, q.1, solution).unwrap();
                    let route_str = route
                        .into_iter()
                        .map(|tr| format!("{}", tr))
                        .collect::<Vec<String>>()
                        .join(" ");
                    // let tree = solution
                    //     .into_iter()
                    //     .map(|(src, tr)| format!("{} {}", src, tr))
                    //     .collect::<Vec<String>>()
                    //     .join("\n    ");
                    format!(
                        "{}\n  result traverses {} links:\n    {}",
                        query, length, route_str
                    )
                }
            };
            println!("{}", msg)
        }
    }
}
