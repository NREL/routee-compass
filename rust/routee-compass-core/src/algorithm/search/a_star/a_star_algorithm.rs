use super::a_star_frontier::AStarFrontier;
use crate::algorithm::search::edge_traversal::EdgeTraversal;
use crate::algorithm::search::search_error::SearchError;
use crate::algorithm::search::search_tree_branch::SearchTreeBranch;
use crate::algorithm::search::MinSearchTree;
use crate::model::cost::Cost;
use crate::model::frontier::frontier_model::FrontierModel;
use crate::model::road_network::edge_id::EdgeId;
use crate::model::road_network::graph::Graph;
use crate::model::termination::termination_model::TerminationModel;
use crate::model::traversal::state::traversal_state::TraversalState;
use crate::model::traversal::traversal_model::TraversalModel;
use crate::util::read_only_lock::ExecutorReadOnlyLock;
use crate::{algorithm::search::direction::Direction, model::road_network::vertex_id::VertexId};
use priority_queue::PriorityQueue;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLockReadGuard;
use std::time::Instant;

/// run an A* Search over the given directed graph model. traverses links
/// from the source, via the provided direction, to the target. uses the
/// provided traversal model for state updates and link costs. estimates
/// the distance to the destination (the a* heuristic) using the provided
/// cost estimate function.
pub fn run_a_star(
    source: VertexId,
    target: Option<VertexId>,
    directed_graph: Arc<ExecutorReadOnlyLock<Graph>>,
    m: Arc<dyn TraversalModel>,
    frontier_model: Arc<ExecutorReadOnlyLock<Box<dyn FrontierModel>>>,
    termination_model: Arc<ExecutorReadOnlyLock<TerminationModel>>,
) -> Result<MinSearchTree, SearchError> {
    if target.map_or(false, |t| t == source) {
        let empty: HashMap<VertexId, SearchTreeBranch> = HashMap::new();
        return Ok(empty);
    }

    // context for the search (graph, search functions, frontier priority queue)
    let g = directed_graph
        .read()
        .map_err(|e| SearchError::ReadOnlyPoisonError(e.to_string()))?;
    let f = frontier_model
        .read()
        .map_err(|e| SearchError::ReadOnlyPoisonError(e.to_string()))?;
    let t = termination_model
        .read()
        .map_err(|e| SearchError::ReadOnlyPoisonError(e.to_string()))?;

    let mut costs: PriorityQueue<VertexId, std::cmp::Reverse<Cost>> = PriorityQueue::new();
    let mut frontier: HashMap<VertexId, AStarFrontier> = HashMap::new();
    let mut traversal_costs: HashMap<VertexId, Cost> = HashMap::new();
    let mut solution: HashMap<VertexId, SearchTreeBranch> = HashMap::new();

    // setup initial search state
    traversal_costs.insert(source, Cost::ZERO);
    let initial_state = m.initial_state();
    let origin = AStarFrontier {
        vertex_id: source,
        prev_edge_id: None,
        state: initial_state.clone(),
    };

    let origin_cost = match target {
        None => Cost::ZERO,
        Some(target_vertex_id) => h_cost(source, target_vertex_id, &initial_state, &g, &m)?,
    };
    costs.push(source, std::cmp::Reverse(origin_cost));
    frontier.insert(source, origin);

    let start_time = Instant::now();
    let mut iterations = 0;

    loop {
        // handle app-level termination conditions
        if t.terminate_search(&start_time, solution.len(), iterations)? {
            match t.explain_termination(&start_time, solution.len(), iterations) {
                None => {
                    return Err(SearchError::InternalSearchError(format!(
                        "unable to explain termination with start_time, solution_size, iterations: {:?}, {}, {}",
                        &start_time,
                        solution.len(),
                        iterations
                    )))
                }
                Some(msg) => return Err(SearchError::QueryTerminated(msg)),
            }
        }

        // grab the current vertex id, but handle some other termination conditions
        // based on the state of the priority queue and optional search destination
        // - we reach the destination                                       (Ok)
        // - if the set is ever empty and there's no destination            (Ok)
        // - if the set is ever empty and there's a destination             (Err)
        let current_vertex_id = match (costs.pop(), target) {
            (None, Some(target_vertex_id)) => {
                return Err(SearchError::NoPathExists(source, target_vertex_id))
            }
            (None, None) => break,
            (Some((current_v, _)), Some(target_v)) if current_v == target_v => break,
            (Some((current_vertex_id, _)), _) => current_vertex_id,
        };

        let current = frontier.get(&current_vertex_id).cloned().ok_or_else(|| {
            SearchError::InternalSearchError(format!(
                "expected vertex id {} missing from frontier",
                current_vertex_id
            ))
        })?;

        // test for search termination according to the traversal model (Ok/Err)
        if m.terminate_search(&current.state)
            .map_err(SearchError::TraversalModelFailure)?
        {
            break;
        };

        // visit all neighbors of this source vertex
        let neighbor_triplets = g
            .incident_triplets(current.vertex_id, Direction::Forward)
            .map_err(SearchError::GraphError)?;
        for (src_id, edge_id, dst_id) in neighbor_triplets {
            // first make sure we have a valid edge
            let e = g.get_edge(edge_id).map_err(SearchError::GraphError)?;
            if !f.valid_frontier(e, &current.state)? {
                continue;
            }
            let et = EdgeTraversal::new(edge_id, current.prev_edge_id, &current.state, &g, &m)?;
            let current_gscore = traversal_costs
                .get(&src_id)
                .unwrap_or(&Cost::INFINITY)
                .to_owned();
            let tentative_gscore = current_gscore + et.edge_cost();
            let existing_gscore = traversal_costs
                .get(&dst_id)
                .unwrap_or(&Cost::INFINITY)
                .to_owned();
            if tentative_gscore < existing_gscore {
                traversal_costs.insert(dst_id, tentative_gscore);

                let result_state = et.result_state.clone();

                // update solution
                let traversal = SearchTreeBranch {
                    terminal_vertex: src_id,
                    edge_traversal: et,
                };
                solution.insert(dst_id, traversal);

                // update search state
                let f = AStarFrontier {
                    vertex_id: dst_id,
                    prev_edge_id: Some(edge_id),
                    state: result_state,
                };
                let dst_h_cost = match target {
                    None => Cost::ZERO,
                    Some(target_v) => h_cost(dst_id, target_v, &current.state, &g, &m)?,
                };
                let f_score_value = tentative_gscore + dst_h_cost;
                costs.push_increase(f.vertex_id, std::cmp::Reverse(f_score_value));
                frontier.insert(f.vertex_id, f);
            }
        }
        iterations += 1;

        // match (costs.pop(), target) {
        //     (None, Some(target_vertex_id)) => {
        //         Err(SearchError::NoPathExists(source, target_vertex_id))
        //     }
        //     (None, None) => Ok(solution),
        //     Some((current_vertex_id, _)) if current_vertex_id == target => {
        //         break;
        //     }
        //     Some((current_vertex_id, _)) => {

        //     }
        // }
    }
    log::debug!(
        "search iterations: {}, size of search tree: {}",
        iterations,
        solution.len()
    );

    Ok(solution)
}

/// convenience method when origin and destination are specified using
/// edge ids instead of vertex ids. invokes a vertex-oriented search
/// from the out-vertex of the source edge to the in-vertex of the
/// target edge. composes the result with the source and target.
///
/// not tested.
pub fn run_a_star_edge_oriented(
    source: EdgeId,
    target: Option<EdgeId>,
    directed_graph: Arc<ExecutorReadOnlyLock<Graph>>,
    m: Arc<dyn TraversalModel>,
    frontier_model: Arc<ExecutorReadOnlyLock<Box<dyn FrontierModel>>>,
    termination_model: Arc<ExecutorReadOnlyLock<TerminationModel>>,
) -> Result<MinSearchTree, SearchError> {
    // 1. guard against edge conditions (src==dst, src.dst_v == dst.src_v)
    let g = directed_graph
        .read()
        .map_err(|e| SearchError::ReadOnlyPoisonError(e.to_string()))?;
    let source_edge_src_vertex_id = g.src_vertex_id(source)?;
    let source_edge_dst_vertex_id = g.dst_vertex_id(source)?;
    let src_et = EdgeTraversal {
        edge_id: source,
        access_cost: Cost::ZERO,
        traversal_cost: Cost::ZERO,
        result_state: m.initial_state(),
    };
    let src_traversal = SearchTreeBranch {
        terminal_vertex: source_edge_src_vertex_id,
        edge_traversal: src_et,
    };

    match target {
        None => {
            let mut tree: HashMap<VertexId, SearchTreeBranch> = run_a_star(
                source_edge_dst_vertex_id,
                None,
                directed_graph.clone(),
                m.clone(),
                frontier_model.clone(),
                termination_model,
            )?;
            if !tree.contains_key(&source_edge_dst_vertex_id) {
                tree.extend([(source_edge_dst_vertex_id, src_traversal)]);
            }
            Ok(tree)
        }
        Some(target_edge) => {
            let target_edge_src_vertex_id = g.src_vertex_id(target_edge)?;
            let target_edge_dst_vertex_id = g.dst_vertex_id(target_edge)?;

            if source == target_edge {
                let empty: HashMap<VertexId, SearchTreeBranch> = HashMap::new();
                Ok(empty)
            } else if source_edge_dst_vertex_id == target_edge_src_vertex_id {
                // route is simply source -> target
                let init_state = m.initial_state();
                let src_et = EdgeTraversal::new(source, None, &init_state, &g, &m)?;
                let dst_et =
                    EdgeTraversal::new(target_edge, Some(source), &src_et.result_state, &g, &m)?;
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
                    source_edge_dst_vertex_id,
                    Some(target_edge_src_vertex_id),
                    directed_graph.clone(),
                    m.clone(),
                    frontier_model.clone(),
                    termination_model,
                )?;

                if tree.is_empty() {
                    return Err(SearchError::NoPathExists(
                        source_edge_dst_vertex_id,
                        target_edge_src_vertex_id,
                    ));
                }

                let final_state = &tree
                    .get(&target_edge_src_vertex_id)
                    .ok_or_else(|| {
                        SearchError::VertexMissingFromSearchTree(target_edge_src_vertex_id)
                    })?
                    .edge_traversal
                    .result_state;
                let dst_et = EdgeTraversal {
                    edge_id: target_edge,
                    access_cost: Cost::ZERO,
                    traversal_cost: Cost::ZERO,
                    result_state: final_state.to_vec(),
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

                Ok(tree)
            }
        }
    }
}

pub fn h_cost(
    src: VertexId,
    dst: VertexId,
    state: &TraversalState,
    g: &RwLockReadGuard<Graph>,
    m: &Arc<dyn TraversalModel>,
) -> Result<Cost, SearchError> {
    let src_vertex = g.get_vertex(src)?;
    let dst_vertex = g.get_vertex(dst)?;
    let cost_estimate = m.cost_estimate(src_vertex, dst_vertex, state)?;
    Ok(cost_estimate)
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::algorithm::search::backtrack::vertex_oriented_route;
    use crate::model::frontier::default::no_restriction;
    use crate::model::property::edge::Edge;
    use crate::model::property::vertex::Vertex;
    use crate::model::road_network::graph::Graph;
    use crate::model::traversal::default::distance::DistanceModel;
    use crate::model::traversal::traversal_model::TraversalModel;
    use crate::util::unit::DistanceUnit;
    use crate::{model::road_network::edge_id::EdgeId, util::read_only_lock::DriverReadOnlyLock};
    use rayon::prelude::*;

    fn build_mock_graph() -> Graph {
        let vertices = vec![
            Vertex::new(0, 0.0, 0.0),
            Vertex::new(1, 0.0, 0.0),
            Vertex::new(2, 0.0, 0.0),
            Vertex::new(3, 0.0, 0.0),
        ];

        let edges = vec![
            Edge::new(0, 0, 1, 10.0),
            Edge::new(1, 1, 0, 10.0),
            Edge::new(2, 1, 2, 2.0),
            Edge::new(3, 2, 1, 2.0),
            Edge::new(4, 2, 3, 1.0),
            Edge::new(5, 3, 2, 1.0),
            Edge::new(6, 3, 0, 2.0),
            Edge::new(7, 0, 3, 2.0),
        ];

        let mut adj = vec![HashMap::new(); vertices.len()];
        let mut rev = vec![HashMap::new(); vertices.len()];

        for edge in &edges {
            adj[edge.src_vertex_id.0].insert(edge.edge_id, edge.dst_vertex_id);
            rev[edge.dst_vertex_id.0].insert(edge.edge_id, edge.src_vertex_id);
        }

        Graph {
            adj: adj.into_boxed_slice(),
            rev: rev.into_boxed_slice(),
            edges: edges.into_boxed_slice(),
            vertices: vertices.into_boxed_slice(),
        }
    }

    #[test]
    fn test_e2e_queries() {
        // simple box world that exists in a non-euclidean plane that stretches
        // the distances between vertex 0 and 1. tested using a distance cost
        // function. to zero out the cost estimate function, all vertices have a
        // position of (0,0).
        // (0) <---> (1)
        //  ^         ^
        //  |         |
        //  v         v
        // (3) <---> (2)
        // (0) -[0]-> (1) 10 units distance
        // (1) -[1]-> (0) 10 units distance
        // (1) -[2]-> (2) 2 units distance
        // (2) -[3]-> (1) 2 units distance
        // (2) -[4]-> (3) 1 units distance
        // (3) -[5]-> (2) 1 units distance
        // (3) -[6]-> (0) 2 units distance
        // (0) -[7]-> (3) 2 units distance

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
        let graph = build_mock_graph();
        let driver_dg = Arc::new(DriverReadOnlyLock::new(graph));

        let no_restriction: Box<dyn FrontierModel> = Box::new(no_restriction::NoRestriction {});
        let driver_rm = Arc::new(DriverReadOnlyLock::new(TerminationModel::IterationsLimit {
            limit: 20,
        }));
        // let driver_tm = Arc::new(DriverReadOnlyLock::new(dist_tm));
        let driver_fm = Arc::new(DriverReadOnlyLock::new(no_restriction));

        // execute the route search
        let result: Vec<Result<MinSearchTree, SearchError>> = queries
            .clone()
            .into_par_iter()
            .map(|(o, d, _expected)| {
                let dg_inner = Arc::new(driver_dg.read_only());
                let dist_tm: Arc<dyn TraversalModel> =
                    Arc::new(DistanceModel::new(DistanceUnit::Meters));
                let fm_inner = Arc::new(driver_fm.read_only());
                let rm_inner = Arc::new(driver_rm.read_only());
                run_a_star(o, Some(d), dg_inner, dist_tm, fm_inner, rm_inner)
            })
            .collect();

        // review the search results, confirming that the route result matches the expected route
        for (r, (o, d, expected_route)) in result.into_iter().zip(queries) {
            let solution = r.unwrap();
            let route = vertex_oriented_route(o, d, &solution).unwrap();
            let route_edges: Vec<EdgeId> = route.iter().map(|r| r.edge_id).collect();
            assert_eq!(
                route_edges, expected_route,
                "route did not match expected: {:?} {:?}",
                route_edges, expected_route
            );
        }
    }
}
