use crate::algorithm::search::Direction;
use crate::algorithm::search::SearchError;
use crate::algorithm::search::SearchInstance;
use crate::algorithm::search::SearchResult;
use crate::algorithm::search::SearchTreeBranch;
use crate::model::label::Label;
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
    log::debug!(
        "run_vertex_oriented: source: {}, target: {:?}, direction: {:?}",
        source,
        target,
        direction
    );
    if target == Some(source) {
        return Ok(SearchResult::default());
    }

    // context for the search (graph, search functions, frontier priority queue)
    let mut costs: InternalPriorityQueue<Label, ReverseCost> = InternalPriorityQueue::default();
    let mut traversal_costs: HashMap<Label, Cost> = HashMap::new();
    let mut solution: HashMap<Label, SearchTreeBranch> = HashMap::new();

    // setup initial search state
    let initial_state = si.state_model.initial_state()?;
    let inital_label = si
        .label_model
        .label_from_state(source, &initial_state, &si.state_model)?;
    traversal_costs.insert(inital_label.clone(), Cost::ZERO);
    let origin_cost = match target {
        None => Cost::ZERO,
        Some(target) => {
            let cost_est = si.estimate_traversal_cost(source, target, &initial_state)?;
            Cost::new(cost_est.as_f64() * weight_factor.unwrap_or(Cost::ONE).as_f64())
        }
    };
    costs.push(inital_label, origin_cost.into());

    let start_time = Instant::now();
    let mut iterations = 0;

    loop {
        si.termination_model
            .test(&start_time, solution.len(), iterations)?;

        let (current_label, current_vertex_id) =
            match advance_search(&mut costs, source, target, &solution)? {
                None => break,
                Some((label, vertex_id)) => (label, vertex_id),
            };

        let last_edge_id = if current_vertex_id == source {
            None
        } else {
            solution
                .get(&current_label)
                .map(|branch| branch.edge_traversal.edge_id)
        };

        // grab the current state from the solution
        let current_state = if current_vertex_id == source {
            initial_state.clone()
        } else {
            solution
                .get(&current_label)
                .ok_or_else(|| {
                    SearchError::InternalError(format!(
                        "expected label {:?} missing from solution",
                        current_label
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
            let terminal_label = si.label_model.label_from_state(
                terminal_vertex_id,
                &current_state,
                &si.state_model,
            )?;
            let key_vertex_id = direction.tree_key_vertex_id(e);

            let previous_edge = match last_edge_id {
                Some(edge_id) => Some(si.graph.get_edge(&edge_id)?),
                None => None,
            };
            let valid_frontier = {
                si.frontier_model.valid_frontier(
                    e,
                    previous_edge,
                    &current_state,
                    &si.state_model,
                )?
            };
            if !valid_frontier {
                continue;
            }

            let et =
                direction.perform_edge_traversal(*edge_id, last_edge_id, &current_state, si)?;

            let key_label = si.label_model.label_from_state(
                key_vertex_id,
                &et.result_state,
                &si.state_model,
            )?;

            let current_gscore = traversal_costs
                .get(&terminal_label)
                .unwrap_or(&Cost::INFINITY)
                .to_owned();
            let tentative_gscore = current_gscore + et.total_cost();
            let existing_gscore = traversal_costs
                .get(&key_label)
                .unwrap_or(&Cost::INFINITY)
                .to_owned();
            if tentative_gscore < existing_gscore {
                traversal_costs.insert(key_label.clone(), tentative_gscore);

                // update solution
                let traversal = SearchTreeBranch {
                    terminal_label,
                    edge_traversal: et.clone(),
                };
                solution.insert(key_label.clone(), traversal);

                let dst_h_cost = match target {
                    None => Cost::ZERO,
                    Some(target_v) => {
                        let cost_est =
                            si.estimate_traversal_cost(key_vertex_id, target_v, &current_state)?;
                        Cost::new(cost_est.as_f64() * weight_factor.unwrap_or(Cost::ONE).as_f64())
                    }
                };
                let f_score_value = tentative_gscore + dst_h_cost;
                costs.push_increase(key_label, f_score_value.into());
            }
        }
        iterations += 1;
    }
    log::debug!(
        "search iterations: {}, size of search tree: {}",
        iterations,
        solution.len()
    );

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
    // For now, convert to vertex-oriented search and use compatibility layer
    let _e1_src = si.graph.src_vertex_id(&source)?;
    let e1_dst = si.graph.dst_vertex_id(&source)?;

    match target {
        None => run_vertex_oriented(e1_dst, None, direction, weight_factor, si),
        Some(target_edge) => {
            let e2_src = si.graph.src_vertex_id(&target_edge)?;
            let _e2_dst = si.graph.dst_vertex_id(&target_edge)?;

            if source == target_edge {
                Ok(SearchResult::default())
            } else {
                run_vertex_oriented(e1_dst, Some(e2_src), direction, weight_factor, si)
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
/// * `cost`   - queue of priority-ranked labels for exploration
/// * `source` - search source vertex
/// * `target` - optional search destination
///
/// # Results
/// The next label and vertex to search. None if the queue has been exhausted in a search with no
/// destination, or we have reached our destination.
/// An error if no path exists for a search that includes a destination.
fn advance_search(
    cost: &mut InternalPriorityQueue<Label, ReverseCost>,
    source: VertexId,
    target: Option<VertexId>,
    solution: &HashMap<Label, SearchTreeBranch>,
) -> Result<Option<(Label, VertexId)>, SearchError> {
    match (cost.pop(), target) {
        (None, Some(target_vertex_id)) => {
            // for debugging purposes, we write the current state of the search to a file
            let outfile = format!("search_no_path_{}_{}.json", source.0, target_vertex_id.0);
            std::fs::write(outfile, serialize_search_tree(solution).unwrap()).unwrap();

            Err(SearchError::NoPathExistsBetweenVertices(
                source,
                target_vertex_id,
                solution.len(),
            ))
        }
        (None, None) => Ok(None),
        (Some((current_label, _)), Some(target_v)) if current_label.vertex_id() == target_v => {
            Ok(None)
        }
        (Some((current_label, _)), _) => {
            Ok(Some((current_label.clone(), current_label.vertex_id())))
        }
    }
}

fn serialize_search_tree(
    tree: &HashMap<Label, SearchTreeBranch>,
) -> Result<String, serde_json::Error> {
    let string_map: HashMap<String, SearchTreeBranch> = tree
        .iter()
        .map(|(label, branch)| (label.to_string(), branch.clone()))
        .collect();
    serde_json::to_string(&string_map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithm::search::backtrack::label_oriented_route;
    use crate::algorithm::search::MinSearchTree;
    use crate::model::access::default::NoAccessModel;
    use crate::model::cost::CostAggregation;
    use crate::model::cost::CostModel;
    use crate::model::cost::VehicleCostRate;
    use crate::model::frontier::default::no_restriction::NoRestriction;

    use crate::model::label::default::vertex_label_model::VertexLabelModel;
    use crate::model::map::MapModel;
    use crate::model::map::MapModelConfig;
    use crate::model::network::edge_id::EdgeId;
    use crate::model::network::graph::Graph;
    use crate::model::network::Edge;
    use crate::model::network::Vertex;
    use crate::model::state::StateModel;
    use crate::model::termination::TerminationModel;
    use crate::model::traversal::default::distance::DistanceTraversalModel;
    use crate::model::traversal::TraversalModel;
    use crate::util::compact_ordered_hash_map::CompactOrderedHashMap;
    use rayon::prelude::*;
    use std::sync::Arc;
    use uom::si::f64::Length;

    fn build_mock_graph() -> Graph {
        let vertices = vec![
            Vertex::new(0, 0.0, 0.0),
            Vertex::new(1, 0.0, 0.0),
            Vertex::new(2, 0.0, 0.0),
            Vertex::new(3, 0.0, 0.0),
        ];

        let edges = vec![
            Edge::new(0, 0, 1, Length::new::<uom::si::length::kilometer>(10.0)),
            Edge::new(1, 1, 0, Length::new::<uom::si::length::kilometer>(10.0)),
            Edge::new(2, 1, 2, Length::new::<uom::si::length::kilometer>(2.0)),
            Edge::new(3, 2, 1, Length::new::<uom::si::length::kilometer>(2.0)),
            Edge::new(4, 2, 3, Length::new::<uom::si::length::kilometer>(1.0)),
            Edge::new(5, 3, 2, Length::new::<uom::si::length::kilometer>(1.0)),
            Edge::new(6, 3, 0, Length::new::<uom::si::length::kilometer>(2.0)),
            Edge::new(7, 0, 3, Length::new::<uom::si::length::kilometer>(2.0)),
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
        let traversal_model = Arc::new(DistanceTraversalModel {});

        // setup the graph, traversal model, and a* heuristic to be shared across the queries in parallel
        // these live in the "driver" process and are passed as read-only memory to each executor process
        let state_model = Arc::new(
            StateModel::empty()
                .register(
                    traversal_model.clone().input_features(),
                    traversal_model.clone().output_features(),
                )
                .unwrap(),
        );
        let cost_model = CostModel::new(
            // vec![(String::from("distance"), 0usize)],
            Arc::new(HashMap::from([(String::from("trip_distance"), 1.0)])),
            Arc::new(HashMap::from([(
                String::from("trip_distance"),
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
            traversal_model: traversal_model.clone(),
            access_model: Arc::new(NoAccessModel {}),
            cost_model: Arc::new(cost_model),
            frontier_model: Arc::new(NoRestriction {}),
            termination_model: Arc::new(TerminationModel::IterationsLimit { limit: 20 }),
            label_model: Arc::new(VertexLabelModel {}),
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
            let route = label_oriented_route(o, d, &solution).unwrap();
            let route_edges: Vec<EdgeId> = route.iter().map(|r| r.edge_id).collect();
            assert_eq!(
                route_edges, expected_route,
                "route did not match expected: {:?} {:?}",
                route_edges, expected_route
            );
        }
    }
}
