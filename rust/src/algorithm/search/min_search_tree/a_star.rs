use std::collections::{BinaryHeap, HashMap, HashSet};
use std::sync::RwLockReadGuard;

use crate::model::cost::cost::Cost;
use crate::model::cost::function::{
    CostEstFn, CostEstimateFunction, CostFn, EdgeEdgeMetricFn, EdgeMetricFn,
};
use crate::model::traversal::traversal_model::TraversalModel;
use crate::util::read_only_lock::ExecutorReadOnlyLock;
use crate::{
    algorithm::search::min_search_tree::{direction::Direction, edge_frontier::EdgeFrontier},
    model::graph::{directed_graph::DirectedGraph, edge_id::EdgeId, vertex_id::VertexId},
};

use std::sync::{Arc, RwLock};

use super::search_error::SearchError;
use super::solution::Solution;
use super::vertex_frontier::VertexFrontier;

type MinSearchTree = HashMap<VertexId, Solution>;

pub fn run_a_star<S: Sync + Send + Eq + Copy + Clone>(
    directed_graph: Arc<ExecutorReadOnlyLock<&dyn DirectedGraph>>,
    direction: Direction,
    source: VertexId,
    target: VertexId,
    traversal_model: Arc<ExecutorReadOnlyLock<&dyn TraversalModel<State = S>>>,
    cost_estimate_fn: Arc<ExecutorReadOnlyLock<&dyn CostEstimateFunction>>,
) -> Result<MinSearchTree, SearchError> {
    // context for the search (graph, search functions, frontier priority queue)
    let g = directed_graph.read().unwrap();
    let m = traversal_model.read().unwrap();
    let c = cost_estimate_fn.read().unwrap();
    let mut open_set: BinaryHeap<VertexFrontier<S>> = BinaryHeap::new();
    let mut came_from: HashMap<VertexId, VertexId> = HashMap::new();
    let mut g_score: HashMap<VertexId, Cost> = HashMap::new();
    let mut h_score: HashMap<VertexId, Cost> = HashMap::new();

    g_score.insert(source, Cost::ZERO);
    h_score.insert(source, h_cost(source, target, &c, &g)?);
    open_set.push(VertexFrontier {
        vertex_id: source,
        prev_edge_id: None,
        state: m.initial_state()?,
        cost: Cost::ZERO,
    });

    loop {
        match open_set.pop() {
            None => break,
            Some(current) if current.vertex_id == target => break,
            Some(current) => {
                let triplets = g
                    .incident_triplets(current.vertex_id, direction)
                    .map_err(SearchError::GraphCorrectnessFailure)?;

                // notes
                //  a* has different invariants than Dijkstra's i realize, since we are no
                //  longer holding to the min ordered heap, we may visit a node multiple times.
                //  hopped over to the wikipedia pseudocode to get this right for a*. we will
                //  want to make separate .rs files for other searches and possibly unify shared
                //  function logic in a search ops module.

                // hey, also, let's make sure we're doing all the fancy stuff we need to cover
                // Edge => Costs, Edge => Edge => Costs, State, etc.

                for (src_id, edge_id, dst_id) in triplets {
                    let tentative_score = current.cost + h_cost(src_id, target, &c, &g)?;
                    let neighbor_score = g_score
                        .get(&dst_id)
                        .ok_or(SearchError::InternalSearchError)?;
                    if &tentative_score < neighbor_score {
                        ////     // This path to neighbor is better than any previous one. Record it!
                        ////     cameFrom[neighbor] := current
                        ////     gScore[neighbor] := tentative_gScore
                        ////     fScore[neighbor] := tentative_gScore + h(neighbor)
                        ////     if neighbor not in openSet
                        ////         openSet.add(neighbor)
                    }
                }
            }
        }

        // todo: explore the leading frontier edge
        // match heap.pop() {
        //     None => break,
        //     Some(f) => {
        //         // todo: transition frontier to a vertex-oriented model? review pseudocode
        //         //   - needs to apply Edge => Edge => Cost function
        //         //   - do we need to store travel time explicitly on the frontier to apply cost_estimate_fn?
        //         //   - should cost_estimate_fn come from the TraversalModel?
        //         //     - coupled with the State, we could know how to store/retrieve the relevant g score + apply the h score
        //         //     - h scores could then leverage same exact cost function easily
        //         // todo: evaluate h score (cost_estimate_fn) in expand
        //         // todo: finalize Solution model, assign results

        //         let expand_vertex_id = g
        //             .incident_vertex(f.edge_id, direction)
        //             .map_err(SearchError::GraphCorrectnessFailure)?;

        //         let terminate_for_user = m
        //             .terminate_search(&f)
        //             .map_err(SearchError::TraversalModelFailure)?;

        //         let terminate_for_target = target.map(|v| v == expand_vertex_id).unwrap_or(false);
        //         if terminate_for_user || terminate_for_target {
        //             break;
        //         }

        //         let this_frontier =
        //             expand(expand_vertex_id, f.prev_edge_id, direction, f.state, &m, &g)?;
        //         for next_f in this_frontier {
        //             heap.push(next_f);
        //         }
        //     }
        // }
    }

    return Result::Ok(HashMap::new());
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

fn expand<S: Sync + Send + Eq + Copy + Clone>(
    vertex_id: VertexId,
    prev_edge_id: Option<EdgeId>,
    direction: Direction,
    prev_state: S,
    // traversal_model: Arc<RwLock<dyn TraversalModel<State = S>>>,
    // directed_graph: Arc<RwLock<dyn DirectedGraph>>,
    m: RwLockReadGuard<&dyn TraversalModel<State = S>>,
    g: RwLockReadGuard<&dyn DirectedGraph>,
) -> Result<Vec<EdgeFrontier<S>>, SearchError> {
    // find in or out edges from this vertex id
    let initial_edges = g
        .incident_edges(vertex_id, direction)
        .map_err(SearchError::GraphCorrectnessFailure)?;

    let mut expanded: Vec<EdgeFrontier<S>> = vec![];

    for edge_id in initial_edges {
        let edge = g
            .edge_attr(edge_id)
            .map_err(SearchError::GraphCorrectnessFailure)?;

        let (access_cost, access_state);
        (access_cost, access_state) = match prev_edge_id {
            Some(prev_e) => {
                let prev_edge = g
                    .edge_attr(prev_e)
                    .map_err(SearchError::GraphCorrectnessFailure)?;
                m.access_cost(&prev_edge, &edge, &prev_state)
            }
            None => Ok((Cost::ZERO, prev_state)),
        }
        .map_err(SearchError::TraversalModelFailure)?;

        let (traversal_cost, traversal_state);
        (traversal_cost, traversal_state) = m
            .traversal_cost(&edge, &access_state)
            .map_err(SearchError::TraversalModelFailure)?;

        let initial_frontier = EdgeFrontier {
            edge_id,
            prev_edge_id,
            state: traversal_state,
            cost: access_cost + traversal_cost,
        };

        expanded.push(initial_frontier);
    }

    return Ok(expanded);
}

#[cfg(test)]
mod tests {
    use crate::{
        model::{
            cost::cost_error::CostError,
            graph::graph_error::GraphError,
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

    struct TestDG;
    impl DirectedGraph for TestDG {
        fn edge_attr(&self, edge_id: EdgeId) -> Result<Edge, GraphError> {
            Ok(Edge {
                start_node: VertexId(0),
                end_node: VertexId(1),
                road_class: RoadClass(0),
                free_flow_speed_seconds: CmPerSecond(80),
                distance_centimeters: Centimeters(1000),
                grade_millis: Millis(0),
            })
        }
        fn vertex_attr(&self, vertex_id: VertexId) -> Result<Vertex, GraphError> {
            Ok(Vertex {
                x: Ordinate(0),
                y: Ordinate(0),
            })
        }
        fn out_edges(&self, src: VertexId) -> Result<Vec<EdgeId>, GraphError> {
            Ok(vec![EdgeId(1)])
        }
        fn in_edges(&self, src: VertexId) -> Result<Vec<EdgeId>, GraphError> {
            Ok(vec![EdgeId(1)])
        }
        fn src_vertex(&self, edge_id: EdgeId) -> Result<VertexId, GraphError> {
            Ok(VertexId(1))
        }
        fn dst_vertex(&self, edge_id: EdgeId) -> Result<VertexId, GraphError> {
            Ok(VertexId(1))
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

        let driver_dg_obj = TestDG;
        let driver_dg = Arc::new(DriverReadOnlyLock::new(
            &driver_dg_obj as &dyn DirectedGraph,
        ));
        let driver_tm_obj = TestModel;
        let driver_tm = Arc::new(DriverReadOnlyLock::new(
            &driver_tm_obj as &dyn TraversalModel<State = i64>,
        ));

        // todo:
        // - finish sending correct types to run_a_star, below
        // - create an iterator of 4 vertex/vertex pairs as queries
        // - setup the road network to play well with the test queries
        // - confirm that we can parallelize queries with shared memory
        // - handle result of fork with a join and test of Result<>

        let queries: Vec<(VertexId, VertexId)> = vec![
            (VertexId(0), VertexId(1)),
            (VertexId(2), VertexId(3)),
            (VertexId(4), VertexId(5)),
            (VertexId(6), VertexId(7)),
        ];

        let result: Vec<Result<MinSearchTree, SearchError>> = queries
            .into_par_iter()
            .map(|(o, d)| {
                let dg_inner = Arc::new(driver_dg.read_only());
                let tm_inner = Arc::new(driver_tm.read_only());
                let cost_inner = Arc::new(driver_cf.read_only());
                run_a_star(dg_inner, Direction::Forward, o, d, tm_inner, cost_inner)
            })
            .collect();

        for r in result {
            let msg = match r {
                Ok(m) => m.len().to_string(),
                Err(e) => e.to_string(),
            };
            println!("{}", msg)
        }
    }
}
