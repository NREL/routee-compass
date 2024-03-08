use crate::algorithm::search::edge_traversal::EdgeTraversal;
use crate::algorithm::search::search_error::SearchError;
use crate::algorithm::search::search_instance::SearchInstance;
use crate::algorithm::search::search_result::SearchResult;
use crate::algorithm::search::search_tree_branch::SearchTreeBranch;
use crate::model::road_network::edge_id::EdgeId;
use crate::model::road_network::vertex_id::VertexId;
use crate::model::unit::cost::ReverseCost;
use crate::model::unit::Cost;
use crate::util::priority_queue::InternalPriorityQueue;
use priority_queue::PriorityQueue;
use std::collections::HashMap;
use std::time::Instant;

/// run an A* Search over the given directed graph model. traverses links
/// from the source, via the provided direction, to the target. uses the
/// provided traversal model for state updates and link costs. estimates
/// the distance to the destination (the a* heuristic) using the provided
/// cost estimate function.
pub fn run_a_star(
    source: VertexId,
    target: Option<VertexId>,
    si: &SearchInstance,
) -> Result<SearchResult, SearchError> {
    if target.map_or(false, |t| t == source) {
        return Ok(SearchResult::default());
    }

    // context for the search (graph, search functions, frontier priority queue)
    let mut costs: InternalPriorityQueue<VertexId, ReverseCost> =
        InternalPriorityQueue(PriorityQueue::new());
    let mut traversal_costs: HashMap<VertexId, Cost> = HashMap::new();
    let mut solution: HashMap<VertexId, SearchTreeBranch> = HashMap::new();

    // setup initial search state
    traversal_costs.insert(source, Cost::ZERO);
    let initial_state = si.state_model.initial_state()?;
    let origin_cost = match target {
        None => Cost::ZERO,
        Some(target) => si.estimate_traversal_cost(source, target, &initial_state)?,
    };
    costs.push(source, origin_cost.into());

    let start_time = Instant::now();
    let mut iterations = 0;

    loop {
        // handle app-level termination conditions
        let should_terminate =
            si.termination_model
                .terminate_search(&start_time, solution.len(), iterations)?;

        if should_terminate {
            let explanation =
                si.termination_model
                    .explain_termination(&start_time, solution.len(), iterations);
            match explanation {
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

        let previous_edge = if current_vertex_id == source {
            None
        } else {
            let edge_id = solution
                .get(&current_vertex_id)
                .ok_or_else(|| {
                    SearchError::InternalSearchError(format!(
                        "expected vertex id {} missing from solution",
                        current_vertex_id
                    ))
                })?
                .edge_traversal
                .edge_id;
            let edge = si
                .directed_graph
                .get_edge(edge_id)
                .map_err(SearchError::GraphError)?;
            Some(edge)
        };

        // grab the current state from the solution
        let current_state = if current_vertex_id == source {
            initial_state.clone()
        } else {
            solution
                .get(&current_vertex_id)
                .ok_or_else(|| {
                    SearchError::InternalSearchError(format!(
                        "expected vertex id {} missing from solution",
                        current_vertex_id
                    ))
                })?
                .edge_traversal
                .result_state
                .clone()
        };

        // visit all neighbors of this source vertex
        for edge_id in si.directed_graph.out_edges_iter(current_vertex_id)? {
            let e = si.directed_graph.get_edge(*edge_id)?;
            let src_id = e.src_vertex_id;
            let dst_id = e.dst_vertex_id;

            if !si.frontier_model.valid_frontier(
                e,
                &current_state,
                previous_edge,
                &si.state_model,
            )? {
                continue;
            }
            let et = EdgeTraversal::perform_traversal(
                *edge_id,
                previous_edge.map(|pe| pe.edge_id),
                &current_state,
                si,
            )?;
            let current_gscore = traversal_costs
                .get(&src_id)
                .unwrap_or(&Cost::INFINITY)
                .to_owned();
            let tentative_gscore = current_gscore + et.total_cost();
            let existing_gscore = traversal_costs
                .get(&dst_id)
                .unwrap_or(&Cost::INFINITY)
                .to_owned();
            if tentative_gscore < existing_gscore {
                traversal_costs.insert(dst_id, tentative_gscore);

                // update solution
                let traversal = SearchTreeBranch {
                    terminal_vertex: src_id,
                    edge_traversal: et,
                };
                solution.insert(dst_id, traversal);

                let dst_h_cost = match target {
                    None => Cost::ZERO,
                    Some(target_v) => {
                        si.estimate_traversal_cost(dst_id, target_v, &current_state)?
                    }
                };
                let f_score_value = tentative_gscore + dst_h_cost;
                costs.push_increase(dst_id, f_score_value.into());
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
pub fn run_a_star_edge_oriented(
    source: EdgeId,
    target: Option<EdgeId>,
    si: &SearchInstance,
) -> Result<SearchResult, SearchError> {
    // 1. guard against edge conditions (src==dst, src.dst_v == dst.src_v)
    let e1_src = si.directed_graph.src_vertex_id(source)?;
    let e1_dst = si.directed_graph.dst_vertex_id(source)?;
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
            } = run_a_star(e1_dst, None, si)?;
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
            let e2_src = si.directed_graph.src_vertex_id(target_edge)?;
            let e2_dst = si.directed_graph.dst_vertex_id(target_edge)?;

            if source == target_edge {
                Ok(SearchResult::default())
            } else if e1_dst == e2_src {
                // route is simply source -> target
                let init_state = si.state_model.initial_state()?;
                let src_et = EdgeTraversal::perform_traversal(source, None, &init_state, si)?;
                let dst_et = EdgeTraversal::perform_traversal(
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
                } = run_a_star(e1_dst, Some(e2_src), si)?;

                if tree.is_empty() {
                    return Err(SearchError::NoPathExists(e1_dst, e2_src));
                }

                let final_state = &tree
                    .get(&e2_src)
                    .ok_or_else(|| SearchError::VertexMissingFromSearchTree(e2_src))?
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithm::search::backtrack::vertex_oriented_route;
    use crate::algorithm::search::MinSearchTree;
    use crate::model::cost::cost_aggregation::CostAggregation;
    use crate::model::cost::cost_model::CostModel;
    use crate::model::cost::vehicle::vehicle_cost_rate::VehicleCostRate;
    use crate::model::frontier::default::no_restriction::NoRestriction;
    use crate::model::property::edge::Edge;
    use crate::model::property::vertex::Vertex;
    use crate::model::road_network::edge_id::EdgeId;
    use crate::model::road_network::graph::Graph;
    use crate::model::state::state_feature::StateFeature;
    use crate::model::state::state_model::StateModel;
    use crate::model::termination::termination_model::TerminationModel;
    use crate::model::traversal::default::distance_traversal_model::DistanceTraversalModel;
    use crate::model::unit::{Distance, DistanceUnit};
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
        // let graph = build_mock_graph();
        // let driver_dg = Arc::new(DriverReadOnlyLock::new(graph));

        // // let no_restriction: Arc<dyn FrontierModel> = Arc::new(no_restriction::NoRestriction {});
        // let driver_rm = Arc::new(DriverReadOnlyLock::new(TerminationModel::IterationsLimit {
        //     limit: 20,
        // }));

        let state_model = Arc::new(
            StateModel::empty()
                .extend(vec![(
                    String::from("distance"),
                    StateFeature::Distance {
                        distance_unit: DistanceUnit::Kilometers,
                        initial: Distance::new(0.0),
                    },
                )])
                .unwrap(),
        );
        let cost_model = CostModel::new(
            // vec![(String::from("distance"), 0usize)],
            Arc::new(HashMap::from([(String::from("distance"), 1.0)])),
            state_model.clone(),
            Arc::new(HashMap::from([(
                String::from("distance"),
                VehicleCostRate::Raw,
            )])),
            Arc::new(HashMap::new()),
            CostAggregation::Sum,
        )
        .unwrap();
        let si = SearchInstance {
            directed_graph: Arc::new(build_mock_graph()),
            state_model: state_model.clone(),
            traversal_model: Arc::new(DistanceTraversalModel::new(DistanceUnit::Meters)),
            cost_model,
            frontier_model: Arc::new(NoRestriction {}),
            termination_model: Arc::new(TerminationModel::IterationsLimit { limit: 20 }),
        };

        // execute the route search
        let result: Vec<Result<MinSearchTree, SearchError>> = queries
            .clone()
            .into_par_iter()
            .map(|(o, d, _expected)| {
                // let dg_inner = Arc::new(driver_dg.read_only());
                // let dist_tm: Arc<dyn TraversalModel> =
                //     Arc::new(DistanceTraversalModel::new(DistanceUnit::Meters));
                // let dist_um: CostModel = CostModel::new(
                //     vec![(String::from("distance"), 0usize)],
                //     state_model,
                //     Arc::new(HashMap::from([(
                //         String::from("distance"),
                //         VehicleCostRate::Raw,
                //     )])),
                //     Arc::new(HashMap::new()),
                //     CostAggregation::Sum,
                // )?;
                // let fm_inner = Arc::new(NoRestriction {});
                // let rm_inner = Arc::new(driver_rm.read_only());
                run_a_star(o, Some(d), &si).map(|search_result| search_result.tree)
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
