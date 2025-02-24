use crate::algorithm::search::Direction;
use crate::algorithm::search::EdgeTraversal;
use crate::algorithm::search::SearchError;
use crate::algorithm::search::SearchInstance;
use crate::algorithm::search::SearchResult;
use crate::algorithm::search::SearchTreeBranch;
use crate::model::network::edge_id::EdgeId;
use crate::model::network::vertex_id::VertexId;
use crate::model::unit::AsF64;
use crate::model::unit::Cost;
use crate::model::unit::ReverseCost;
use crate::util::priority_queue::InternalPriorityQueue;

use std::collections::HashMap;
use std::time::Instant;

/// run an A* Search over the given directed graph model. traverses links
/// from the source, via the provided direction, to the target. uses the
/// provided traversal model for state updates and link costs. estimates
/// the distance to the destination (the a* heuristic) using the provided
/// cost estimate function.
pub fn run_vertex_oriented(
    source: VertexId,
    target: Option<VertexId>,
    direction: &Direction,
    weight_factor: Option<Cost>,
    si: &SearchInstance,
) -> Result<SearchResult, SearchError> {
    if target == Some(source) {
        return Ok(SearchResult::default());
    }

    // context for the search (graph, search functions, frontier priority queue)
    let mut costs: InternalPriorityQueue<VertexId, ReverseCost> = InternalPriorityQueue::default();
    let mut traversal_costs: HashMap<VertexId, Cost> = HashMap::new();
    let mut solution: HashMap<VertexId, SearchTreeBranch> = HashMap::new();

    // setup initial search state
    traversal_costs.insert(source, Cost::ZERO);
    let initial_state = si.state_model.initial_state()?;
    let origin_cost = match target {
        None => Cost::ZERO,
        Some(target) => {
            let cost_est = si.estimate_traversal_cost(source, target, &initial_state)?;
            Cost::new(cost_est.as_f64() * weight_factor.unwrap_or(Cost::ONE).as_f64())
        }
    };
    costs.push(source, origin_cost.into());

    let start_time = Instant::now();
    let mut iterations = 0;

    loop {
        si.termination_model
            .test(&start_time, solution.len(), iterations)?;

        let current_vertex_id = match advance_search(&mut costs, source, target)? {
            None => break,
            Some(id) => id,
        };

        let last_edge_id = get_last_traversed_edge_id(&current_vertex_id, &source, &solution)?;
        // let last_edge = match last_edge_id {
        //     Some(id) => Some(si.directed_graph.get_edge(&id)?),
        //     None => None,
        // };

        // grab the current state from the solution
        let current_state = if current_vertex_id == source {
            initial_state.clone()
        } else {
            solution
                .get(&current_vertex_id)
                .ok_or_else(|| {
                    SearchError::InternalError(format!(
                        "expected vertex id {} missing from solution",
                        current_vertex_id
                    ))
                })?
                .edge_traversal
                .result_state
                .clone()
        };

        // visit all neighbors of this source vertex
        let incident_edge_iterator = direction.get_incident_edges(&current_vertex_id, si);
        for edge_id in incident_edge_iterator {
            let e = si.graph.get_edge(edge_id)?;

            let terminal_vertex_id = direction.terminal_vertex_id(e);
            let key_vertex_id = direction.tree_key_vertex_id(e);

            let valid_frontier = si.frontier_model.valid_frontier(
                e,
                &current_state,
                &solution,
                direction,
                &si.state_model,
            )?;
            if !valid_frontier {
                continue;
            }
            let et =
                direction.perform_edge_traversal(*edge_id, last_edge_id, &current_state, si)?;
            let current_gscore = traversal_costs
                .get(&terminal_vertex_id)
                .unwrap_or(&Cost::INFINITY)
                .to_owned();
            let tentative_gscore = current_gscore + et.total_cost();
            let existing_gscore = traversal_costs
                .get(&key_vertex_id)
                .unwrap_or(&Cost::INFINITY)
                .to_owned();
            if tentative_gscore < existing_gscore {
                traversal_costs.insert(key_vertex_id, tentative_gscore);

                // update solution
                let traversal = SearchTreeBranch {
                    terminal_vertex: terminal_vertex_id,
                    edge_traversal: et,
                };
                solution.insert(key_vertex_id, traversal);

                let dst_h_cost = match target {
                    None => Cost::ZERO,
                    Some(target_v) => {
                        let cost_est =
                            si.estimate_traversal_cost(key_vertex_id, target_v, &current_state)?;
                        Cost::new(cost_est.as_f64() * weight_factor.unwrap_or(Cost::ONE).as_f64())
                    }
                };
                let f_score_value = tentative_gscore + dst_h_cost;
                costs.push_increase(key_vertex_id, f_score_value.into());
            }
        }
        iterations += 1;
    }
    log::debug!(
        "search iterations: {}, size of search tree: {}",
        iterations,
        solution.len()
    );

    #[cfg(debug_assertions)]
    {
        use std::io::Write;
        use std::path::PathBuf;

        log::debug!("Building flamegraph for search memory usage..");
        let mut flamegraph = allocative::FlameGraphBuilder::default();
        flamegraph.visit_root(&costs);
        flamegraph.visit_root(&traversal_costs);
        flamegraph.visit_root(&solution);
        let output = flamegraph.finish_and_write_flame_graph();

        let search_name = match target {
            None => format!("{}_to_all", source),
            Some(tid) => format!("{}_to_{}", source, tid),
        };

        let outdir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("target")
            .join("flamegraph");

        if !outdir.exists() {
            std::fs::create_dir(&outdir).unwrap();
        }

        let mut flamegraph_file = std::fs::File::create(
            outdir.join(format!("search_memory_flamegraph_{}.out", search_name)),
        )
        .unwrap();
        flamegraph_file.write_all(output.as_bytes()).unwrap();
    }

    let result = SearchResult::new(solution, iterations);
    Ok(result)
}

/// convenience method when origin and destination are specified using
/// edge ids instead of vertex ids. invokes a vertex-oriented search
/// from the out-vertex of the source edge to the in-vertex of the
/// target edge. composes the result with the source and target.
///
/// not tested.
pub fn run_edge_oriented(
    source: EdgeId,
    target: Option<EdgeId>,
    direction: &Direction,
    weight_factor: Option<Cost>,
    si: &SearchInstance,
) -> Result<SearchResult, SearchError> {
    // 1. guard against edge conditions (src==dst, src.dst_v == dst.src_v)
    let e1_src = si.graph.src_vertex_id(&source)?;
    let e1_dst = si.graph.dst_vertex_id(&source)?;
    let src_et = EdgeTraversal {
        edge_id: source,
        access_cost: Cost::ZERO,
        traversal_cost: Cost::ZERO,
        result_state: si.state_model.initial_state()?,
    };
    let src_branch = SearchTreeBranch {
        terminal_vertex: e1_src,
        edge_traversal: src_et,
    };

    match target {
        None => {
            let SearchResult {
                mut tree,
                iterations,
            } = run_vertex_oriented(e1_dst, None, direction, weight_factor, si)?;
            if !tree.contains_key(&e1_dst) {
                tree.extend([(e1_dst, src_branch)]);
            }
            let updated = SearchResult {
                tree,
                iterations: iterations + 1,
            };
            Ok(updated)
        }
        Some(target_edge) => {
            let e2_src = si.graph.src_vertex_id(&target_edge)?;
            let e2_dst = si.graph.dst_vertex_id(&target_edge)?;

            if source == target_edge {
                Ok(SearchResult::default())
            } else if e1_dst == e2_src {
                // route is simply source -> target
                let init_state = si.state_model.initial_state()?;
                let src_et = EdgeTraversal::forward_traversal(source, None, &init_state, si)?;
                let dst_et = EdgeTraversal::forward_traversal(
                    target_edge,
                    Some(source),
                    &src_et.result_state,
                    si,
                )?;
                let src_traversal = SearchTreeBranch {
                    terminal_vertex: e2_src,
                    edge_traversal: dst_et,
                };
                let dst_traversal = SearchTreeBranch {
                    terminal_vertex: e1_src,
                    edge_traversal: src_et,
                };
                let tree = HashMap::from([(e2_dst, src_traversal), (e1_dst, dst_traversal)]);
                let result = SearchResult {
                    tree,
                    iterations: 1,
                };
                return Ok(result);
            } else {
                // run a search and append source/target edges to result
                let SearchResult {
                    mut tree,
                    iterations,
                } = run_vertex_oriented(e1_dst, Some(e2_src), direction, weight_factor, si)?;

                if tree.is_empty() {
                    return Err(SearchError::NoPathExistsBetweenVertices(e1_dst, e2_src));
                }

                let final_state = &tree
                    .get(&e2_src)
                    .ok_or_else(|| {
                        SearchError::InternalError(format!(
                        "resulting tree with {} branches missing vertex {} expected via backtrack",
                        tree.len(),
                        e2_src
                    ))
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
                    terminal_vertex: e2_src,
                    edge_traversal: dst_et,
                };

                // it is possible that the search already found these vertices. one major edge
                // case is when the trip starts with a u-turn.
                if !tree.contains_key(&e1_dst) {
                    tree.extend([(e1_dst, src_branch)]);
                }
                if !tree.contains_key(&e2_dst) {
                    tree.extend([(e2_dst, dst_traversal)]);
                }

                let result = SearchResult {
                    tree,
                    iterations: iterations + 2,
                };
                Ok(result)
            }
        }
    }
}

/// grab the current vertex id, but handle some other termination conditions
/// based on the state of the priority queue and optional search destination
/// - we reach the destination                                       (Ok)
/// - if the set is ever empty and there's no destination            (Ok)
/// - if the set is ever empty and there's a destination             (Err)
///
/// # Arguments
/// * `cost`   - queue of priority-ranked vertices for exploration
/// * `source` - search source vertex
/// * `target` - optional search destination
///
/// # Results
/// The next vertex to search. None if the queue has been exhausted in a search with no
/// destination, or we have reached our destination.
/// An error if no path exists for a search that includes a destination.
fn advance_search(
    cost: &mut InternalPriorityQueue<VertexId, ReverseCost>,
    source: VertexId,
    target: Option<VertexId>,
) -> Result<Option<VertexId>, SearchError> {
    match (cost.pop(), target) {
        (None, Some(target_vertex_id)) => Err(SearchError::NoPathExistsBetweenVertices(
            source,
            target_vertex_id,
        )),
        (None, None) => Ok(None),
        (Some((current_v, _)), Some(target_v)) if current_v == target_v => Ok(None),
        (Some((current_vertex_id, _)), _) => Ok(Some(current_vertex_id)),
    }
}

/// Find the last-traversed edge before reaching this vertex id.
/// The logic is the same for forward and reverse searches but finds
/// a different result because the trees are different.
/// Forward case: find `prev` from v2 in `(v1)-[prev]->(v2)-[next]->(v3)`
/// Reverse case: find `next` from v2 in `(v1)-[prev]->(v2)-[next]->(v3)`
///
/// # Arguments
/// * `this_vertex_id`  - current vertex, v2 in diagram
/// * `first_vertex_id` - source of this search, the origin vertex in a forward
///                       search or the destination vertex in a reverse search
/// * `tree`            - current search solution tree
///
/// # Returns
///
/// The EdgeId for the edge that was traversed to reach this vertex, or None
/// if no edges have yet been traversed.
fn get_last_traversed_edge_id(
    this_vertex_id: &VertexId,
    first_vertex_id: &VertexId,
    tree: &HashMap<VertexId, SearchTreeBranch>,
) -> Result<Option<EdgeId>, SearchError> {
    if this_vertex_id == first_vertex_id {
        Ok(None)
    } else {
        let edge_id = tree
            .get(this_vertex_id)
            .ok_or_else(|| {
                SearchError::InternalError(format!(
                    "expected vertex id {} missing from solution",
                    this_vertex_id
                ))
            })?
            .edge_traversal
            .edge_id;
        Ok(Some(edge_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithm::search::backtrack::vertex_oriented_route;
    use crate::algorithm::search::MinSearchTree;
    use crate::model::access::default::NoAccessModel;
    use crate::model::cost::CostAggregation;
    use crate::model::cost::CostModel;
    use crate::model::cost::VehicleCostRate;
    use crate::model::frontier::default::no_restriction::NoRestriction;

    use crate::model::map::MapModel;
    use crate::model::map::MapModelConfig;
    use crate::model::network::edge_id::EdgeId;
    use crate::model::network::graph::Graph;
    use crate::model::network::Edge;
    use crate::model::network::Vertex;
    use crate::model::state::StateFeature;
    use crate::model::state::StateModel;
    use crate::model::termination::TerminationModel;
    use crate::model::traversal::default::DistanceTraversalModel;
    use crate::model::unit::{Distance, DistanceUnit};
    use crate::util::compact_ordered_hash_map::CompactOrderedHashMap;
    use rayon::prelude::*;
    use std::sync::Arc;

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

        let mut adj = vec![CompactOrderedHashMap::empty(); vertices.len()];
        let mut rev = vec![CompactOrderedHashMap::empty(); vertices.len()];

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

        let graph = Arc::new(build_mock_graph());
        let map_model = Arc::new(MapModel::new(graph.clone(), MapModelConfig::default()).unwrap());

        // setup the graph, traversal model, and a* heuristic to be shared across the queries in parallel
        // these live in the "driver" process and are passed as read-only memory to each executor process
        let state_model = Arc::new(
            StateModel::empty()
                .extend(vec![(
                    String::from("distance"),
                    StateFeature::Distance {
                        distance_unit: DistanceUnit::Kilometers,
                        initial: Distance::from(0.0),
                    },
                )])
                .unwrap(),
        );
        let cost_model = CostModel::new(
            // vec![(String::from("distance"), 0usize)],
            Arc::new(HashMap::from([(String::from("distance"), 1.0)])),
            Arc::new(HashMap::from([(
                String::from("distance"),
                VehicleCostRate::Raw,
            )])),
            Arc::new(HashMap::new()),
            CostAggregation::Sum,
            state_model.clone(),
        )
        .unwrap();
        let si = SearchInstance {
            graph,
            map_model,
            state_model: state_model.clone(),
            traversal_model: Arc::new(DistanceTraversalModel::new(DistanceUnit::Meters)),
            access_model: Arc::new(NoAccessModel {}),
            cost_model: Arc::new(cost_model),
            frontier_model: Arc::new(NoRestriction {}),
            termination_model: Arc::new(TerminationModel::IterationsLimit { limit: 20 }),
        };

        // execute the route search
        let result: Vec<Result<MinSearchTree, SearchError>> = queries
            .clone()
            .into_par_iter()
            .map(|(o, d, _expected)| {
                run_vertex_oriented(o, Some(d), &Direction::Forward, None, &si)
                    .map(|search_result| search_result.tree)
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
