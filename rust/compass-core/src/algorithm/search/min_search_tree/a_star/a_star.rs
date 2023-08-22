use super::a_star_frontier::AStarFrontier;
use super::cost_estimate_function::CostEstimateFunction;
use crate::algorithm::search::edge_traversal::EdgeTraversal;
use crate::algorithm::search::search_error::SearchError;
use crate::algorithm::search::search_tree_branch::SearchTreeBranch;
use crate::model::cost::cost::Cost;
use crate::model::graph::edge_id::EdgeId;
use crate::model::traversal::traversal_model::TraversalModel;
use crate::util::read_only_lock::ExecutorReadOnlyLock;
use crate::{
    algorithm::search::min_search_tree::direction::Direction,
    model::graph::{directed_graph::DirectedGraph, vertex_id::VertexId},
};
use keyed_priority_queue::KeyedPriorityQueue;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::RwLockReadGuard;

type MinSearchTree = HashMap<VertexId, SearchTreeBranch>;

/// run an A* Search over the given directed graph model. traverses links
/// from the source, via the provided direction, to the target. uses the
/// provided traversal model for state updates and link costs. estimates
/// the distance to the destination (the a* heuristic) using the provided
/// cost estimate function.
pub fn run_a_star(
    direction: Direction,
    source: VertexId,
    target: VertexId,
    directed_graph: Arc<ExecutorReadOnlyLock<Box<dyn DirectedGraph>>>,
    traversal_model: Arc<ExecutorReadOnlyLock<Box<dyn TraversalModel>>>,
    cost_estimate_fn: Arc<ExecutorReadOnlyLock<Box<dyn CostEstimateFunction>>>,
) -> Result<MinSearchTree, SearchError> {
    if source == target {
        let empty: HashMap<VertexId, SearchTreeBranch> = HashMap::new();
        return Ok(empty);
    }

    // context for the search (graph, search functions, frontier priority queue)
    let g = directed_graph
        .read()
        .map_err(|e| SearchError::ReadOnlyPoisonError(e.to_string()))?;
    let m = traversal_model
        .read()
        .map_err(|e| SearchError::ReadOnlyPoisonError(e.to_string()))?;
    let c = cost_estimate_fn
        .read()
        .map_err(|e| SearchError::ReadOnlyPoisonError(e.to_string()))?;
    let mut frontier: KeyedPriorityQueue<AStarFrontier, Cost> = KeyedPriorityQueue::new();
    let mut traversal_costs: HashMap<VertexId, Cost> = HashMap::new();
    let mut solution: HashMap<VertexId, SearchTreeBranch> = HashMap::new();

    // setup initial search state
    traversal_costs.insert(source, Cost::ZERO);
    let origin = AStarFrontier {
        vertex_id: source,
        prev_edge_id: None,
        state: m.initial_state(),
    };
    let origin_cost = h_cost(source, target, &c, &g)?;
    frontier.push(origin, -origin_cost);

    // run search loop until we reach the destination, or fail if the set is ever empty
    loop {
        match frontier.pop() {
            None => return Err(SearchError::NoPathExists(source, target)),
            Some((current, _)) if current.vertex_id == target => break,
            Some((current, _))
                if m.terminate_search(&current.state)
                    .map_err(SearchError::TraversalModelFailure)? =>
            {
                break
            }
            Some((current, _)) => {
                let neighbor_triplets = g
                    .incident_triplets(current.vertex_id, direction)
                    .map_err(SearchError::GraphCorrectnessFailure)?;

                for (src_id, edge_id, dst_id) in neighbor_triplets {
                    // first make sure we have a valid edge
                    let e = g
                        .edge_attr(edge_id)
                        .map_err(SearchError::GraphCorrectnessFailure)?;
                    if !m.valid_edge(&e, &current.state)? {
                        continue;
                    }
                    let et =
                        EdgeTraversal::new(edge_id, current.prev_edge_id, &current.state, &g, &m)?;
                    let dst_h_cost = h_cost(dst_id, target, &c, &g)?;
                    let traversal_cost = traversal_costs
                        .get(&src_id)
                        .unwrap_or(&Cost::INFINITY)
                        .to_owned();
                    let tentative_gscore = traversal_cost + et.edge_cost();
                    let existing_gscore = traversal_costs
                        .get(&dst_id)
                        .unwrap_or(&Cost::INFINITY)
                        .to_owned();
                    if tentative_gscore < existing_gscore {
                        traversal_costs.insert(dst_id, tentative_gscore);

                        // update solution
                        let traversal = SearchTreeBranch {
                            terminal_vertex: src_id,
                            edge_traversal: et.clone(),
                        };
                        solution.insert(dst_id, traversal);

                        // update open set

                        let f = AStarFrontier {
                            vertex_id: dst_id,
                            prev_edge_id: Some(edge_id),
                            state: et.result_state,
                        };
                        let f_score_value = tentative_gscore + dst_h_cost;
                        frontier.push(f, -f_score_value);
                    }
                }
            }
        }
    }

    return Ok(solution);
}

/// convenience method when origin and destination are specified using
/// edge ids instead of vertex ids. invokes a vertex-oriented search
/// from the out-vertex of the source edge to the in-vertex of the
/// target edge. composes the result with the source and target.
///
/// not tested.
pub fn run_a_star_edge_oriented(
    direction: Direction,
    source: EdgeId,
    target: EdgeId,
    directed_graph: Arc<ExecutorReadOnlyLock<Box<dyn DirectedGraph>>>,
    traversal_model: Arc<ExecutorReadOnlyLock<Box<dyn TraversalModel>>>,
    cost_estimate_fn: Arc<ExecutorReadOnlyLock<Box<dyn CostEstimateFunction>>>,
) -> Result<MinSearchTree, SearchError> {
    // 1. guard against edge conditions (src==dst, src.dst_v == dst.src_v)
    let g = directed_graph
        .read()
        .map_err(|e| SearchError::ReadOnlyPoisonError(e.to_string()))?;
    let m: RwLockReadGuard<Box<dyn TraversalModel>> = traversal_model
        .read()
        .map_err(|e| SearchError::ReadOnlyPoisonError(e.to_string()))?;
    let source_edge_src_vertex_id = g.src_vertex(source)?;
    let source_edge_dst_vertex_id = g.dst_vertex(source)?;
    let target_edge_src_vertex_id = g.src_vertex(target)?;
    let target_edge_dst_vertex_id = g.dst_vertex(target)?;

    if source == target {
        let empty: HashMap<VertexId, SearchTreeBranch> = HashMap::new();
        return Ok(empty);
    } else if source_edge_dst_vertex_id == target_edge_src_vertex_id {
        // route is simply source -> target
        let init_state = m.initial_state();
        let src_et = EdgeTraversal::new(source, None, &init_state, &g, &m)?;
        let dst_et = EdgeTraversal::new(target, Some(source), &src_et.result_state, &g, &m)?;
        let src_traversal = SearchTreeBranch {
            terminal_vertex: target_edge_src_vertex_id,
            edge_traversal: dst_et,
        };
        let dst_traversal = SearchTreeBranch {
            terminal_vertex: source_edge_src_vertex_id,
            edge_traversal: src_et,
        };
        let tree = HashMap::from([
            (target_edge_dst_vertex_id, src_traversal),
            (source_edge_dst_vertex_id, dst_traversal),
        ]);
        return Ok(tree);
    } else {
        // run a search and append source/target edges to result
        let mut tree: HashMap<VertexId, SearchTreeBranch> = run_a_star(
            direction,
            source_edge_dst_vertex_id,
            target_edge_src_vertex_id,
            directed_graph.clone(),
            traversal_model.clone(),
            cost_estimate_fn.clone(),
        )?;

        if tree.is_empty() {
            return Err(SearchError::NoPathExists(
                source_edge_dst_vertex_id,
                target_edge_src_vertex_id,
            ));
        }

        // append source/target edge traversals to the tree
        // no costs added for now, this would require flipping the order here and
        // passing the search state into the vertex-oriented search function
        // that included the traversal of the initial edge.
        let init_state = m.initial_state();
        let final_state = &tree
            .get(&target_edge_src_vertex_id)
            .ok_or(SearchError::VertexMissingFromSearchTree(
                target_edge_src_vertex_id,
            ))?
            .edge_traversal
            .result_state;
        let src_et = EdgeTraversal {
            edge_id: source,
            access_cost: Cost::ZERO,
            traversal_cost: Cost::ZERO,
            result_state: init_state,
        };
        let dst_et = EdgeTraversal {
            edge_id: target,
            access_cost: Cost::ZERO,
            traversal_cost: Cost::ZERO,
            result_state: final_state.to_vec(),
        };
        let src_traversal = SearchTreeBranch {
            terminal_vertex: source_edge_src_vertex_id,
            edge_traversal: src_et,
        };
        let dst_traversal = SearchTreeBranch {
            terminal_vertex: target_edge_src_vertex_id,
            edge_traversal: dst_et,
        };

        // it is possible that the search already found these vertices. one major edge
        // case is when the trip starts with a u-turn.
        if !tree.contains_key(&source_edge_dst_vertex_id) {
            tree.extend([(source_edge_dst_vertex_id, src_traversal)]);
        }
        if !tree.contains_key(&target_edge_dst_vertex_id) {
            tree.extend([(target_edge_dst_vertex_id, dst_traversal)]);
        }

        return Ok(tree);
    }
}

/// reconstructs a path from a minimum shortest path tree for some source and target vertex
/// directionality travels up from target to source, toward root of the tree, in both the forward
/// and reverse cases.
pub fn backtrack(
    source_id: VertexId,
    target_id: VertexId,
    solution: &HashMap<VertexId, SearchTreeBranch>,
) -> Result<Vec<EdgeTraversal>, SearchError> {
    let mut result: Vec<EdgeTraversal> = vec![];
    let mut visited: HashSet<EdgeId> = HashSet::new();
    let mut this_vertex = target_id.clone();
    loop {
        if this_vertex == source_id {
            break;
        }
        let traversal = solution
            .get(&this_vertex)
            .ok_or(SearchError::VertexMissingFromSearchTree(this_vertex))?;
        let first_visit = visited.insert(traversal.edge_traversal.edge_id);
        if !first_visit {
            return Err(SearchError::LoopInSearchResult(
                traversal.edge_traversal.edge_id,
            ));
        }
        result.push(traversal.edge_traversal.clone());
        this_vertex = traversal.terminal_vertex;
    }
    let reversed = result.into_iter().rev().collect();
    Ok(reversed)
}

/// edge-oriented backtrack method
pub fn backtrack_edges(
    source_id: EdgeId,
    target_id: EdgeId,
    solution: &HashMap<VertexId, SearchTreeBranch>,
    graph: Arc<ExecutorReadOnlyLock<Box<dyn DirectedGraph>>>,
) -> Result<Vec<EdgeTraversal>, SearchError> {
    let g_inner = graph
        .read()
        .map_err(|e| SearchError::ReadOnlyPoisonError(e.to_string()))?;
    let o_v = g_inner
        .src_vertex(source_id)
        .map_err(SearchError::GraphCorrectnessFailure)?;
    let d_v = g_inner
        .dst_vertex(target_id)
        .map_err(SearchError::GraphCorrectnessFailure)?;
    backtrack(o_v, d_v, solution)
}

/// implements the a* heuristic function based on the provided cost estimate function
/// and graph. estimates travel costs between two vertices in the graph.
fn h_cost(
    vertex_id: VertexId,
    target_id: VertexId,
    c: &RwLockReadGuard<Box<dyn CostEstimateFunction>>,
    g: &RwLockReadGuard<Box<dyn DirectedGraph>>,
) -> Result<Cost, SearchError> {
    let src_v = g
        .vertex_attr(vertex_id)
        .map_err(SearchError::GraphCorrectnessFailure)?;
    let dst_v = g
        .vertex_attr(target_id)
        .map_err(SearchError::GraphCorrectnessFailure)?;
    c.cost(src_v, dst_v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::traversal::default_models::distance::DistanceModel;
    use crate::model::traversal::traversal_model::TraversalModel;
    use crate::model::units::Length;
    use crate::test::mocks::TestDG;
    use crate::{
        model::{graph::edge_id::EdgeId, property::vertex::Vertex},
        util::read_only_lock::DriverReadOnlyLock,
    };
    use rayon::prelude::*;
    use uom::si::length::centimeter;

    struct TestCost;
    impl CostEstimateFunction for TestCost {
        fn cost(&self, _src: Vertex, _dst: Vertex) -> Result<Cost, SearchError> {
            Ok(Cost::from(0.0))
        }
    }

    #[test]
    fn test_e2e_queries() {
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
            (
                VertexId(0),
                HashMap::from([(EdgeId(0), VertexId(1)), (EdgeId(7), VertexId(3))]),
            ),
            (
                VertexId(1),
                HashMap::from([(EdgeId(1), VertexId(0)), (EdgeId(2), VertexId(2))]),
            ),
            (
                VertexId(2),
                HashMap::from([(EdgeId(3), VertexId(1)), (EdgeId(4), VertexId(3))]),
            ),
            (
                VertexId(3),
                HashMap::from([(EdgeId(5), VertexId(2)), (EdgeId(6), VertexId(0))]),
            ),
        ]);
        let edge_lengths = HashMap::from([
            (EdgeId(0), Length::new::<centimeter>(10.0)),
            (EdgeId(1), Length::new::<centimeter>(10.0)),
            (EdgeId(2), Length::new::<centimeter>(2.0)),
            (EdgeId(3), Length::new::<centimeter>(2.0)),
            (EdgeId(4), Length::new::<centimeter>(1.0)),
            (EdgeId(5), Length::new::<centimeter>(1.0)),
            (EdgeId(6), Length::new::<centimeter>(2.0)),
            (EdgeId(7), Length::new::<centimeter>(2.0)),
        ]);

        // these are the queries to test the grid world. for each query,
        // we have the vertex pair (source, target) to submit to the
        // search algorithm, and then the expected route traversal vector for each pair.
        // a comment is provided that illustrates each query/expected traversal combination.
        let queries: Vec<(VertexId, VertexId, Vec<EdgeId>)> = vec![
            (
                // 0 -[7]-> 3 -[5]-> 2 -[3]-> 1
                VertexId(0),
                VertexId(1),
                vec![EdgeId(7), EdgeId(5), EdgeId(3)],
            ),
            (
                // 0 -[7]-> 3
                VertexId(0),
                VertexId(3),
                vec![EdgeId(7)],
            ),
            (
                // 1 -[2]-> 2 -[4]-> 3 -[6]-> 0
                VertexId(1),
                VertexId(0),
                vec![EdgeId(2), EdgeId(4), EdgeId(6)],
            ),
            (VertexId(1), VertexId(2), vec![EdgeId(2)]), // 1 -[2]-> 2
            (VertexId(2), VertexId(3), vec![EdgeId(4)]), // 2 -[4]-> 3
        ];

        // setup the graph, traversal model, and a* heuristic to be shared across the queries in parallel
        // these live in the "driver" process and are passed as read-only memory to each executor process
        let driver_dg_obj: Box<dyn DirectedGraph> =
            Box::new(TestDG::new(adj, edge_lengths).unwrap());
        let driver_dg = Arc::new(DriverReadOnlyLock::new(driver_dg_obj));

        let dist_tm: Box<dyn TraversalModel> = Box::new(DistanceModel {});
        let driver_tm = Arc::new(DriverReadOnlyLock::new(dist_tm));
        let driver_cf_obj: Box<dyn CostEstimateFunction> = Box::new(TestCost);
        let driver_cf = Arc::new(DriverReadOnlyLock::new(driver_cf_obj));

        // execute the route search
        let result: Vec<Result<MinSearchTree, SearchError>> = queries
            .clone()
            .into_par_iter()
            .map(|(o, d, _expected)| {
                let dg_inner = Arc::new(driver_dg.read_only());
                let tm_inner = Arc::new(driver_tm.read_only());
                let cost_inner = Arc::new(driver_cf.read_only());
                run_a_star(Direction::Forward, o, d, dg_inner, tm_inner, cost_inner)
            })
            .collect();

        // review the search results, confirming that the route result matches the expected route
        for (r, (o, d, expected_route)) in result.into_iter().zip(queries) {
            let solution = r.unwrap();
            let route = backtrack(o, d, &solution).unwrap();
            let route_edges: Vec<EdgeId> = route.iter().map(|r| r.edge_id).collect();
            assert_eq!(
                route_edges, expected_route,
                "route did not match expected: {:?} {:?}",
                route_edges, expected_route
            );
        }
    }
}
