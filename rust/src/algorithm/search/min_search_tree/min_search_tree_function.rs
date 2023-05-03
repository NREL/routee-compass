use std::collections::{BinaryHeap, HashMap, HashSet};
use std::sync::RwLockReadGuard;

use crate::model::cost::cost::Cost;
use crate::model::cost::function::{CostEstFn, CostFn, EdgeEdgeMetricFn, EdgeMetricFn};
use crate::model::traversal::traversal_model::TraversalModel;
use crate::{
    algorithm::search::min_search_tree::{direction::Direction, frontier::Frontier},
    model::graph::{directed_graph::DirectedGraph, edge_id::EdgeId, vertex_id::VertexId},
};

use std::sync::{Arc, RwLock};

use super::search_error::SearchError;
use super::solution::Solution;

type MinSearchTree = im::HashMap<VertexId, Solution>;

pub fn run_search<S: Eq + Clone + Copy>(
    directed_graph: Arc<RwLock<dyn DirectedGraph>>,
    direction: Direction,
    source: VertexId,
    target: Option<VertexId>,
    traversal_model: Arc<RwLock<dyn TraversalModel<State = S>>>,
    cost_estimate_fn: Option<Arc<RwLock<CostEstFn>>>,
) -> Result<MinSearchTree, SearchError> {
    if target.is_none() && cost_estimate_fn.is_none() {
        return Result::Err(SearchError::DistanceHeuristicWithNoTarget);
    }

    // context for the search (graph, search functions, frontier priority queue)
    let g = directed_graph.read().unwrap();
    let m = traversal_model.read().unwrap();
    let mut heap: BinaryHeap<Frontier<S>> = BinaryHeap::new();
    let mut tree: HashMap<VertexId, Frontier<S>> = HashMap::new();
    let mut expanded: HashSet<VertexId> = HashSet::new();

    let initial_state = m.initial_state()?;
    let initial_frontier = expand(source, None, direction, initial_state, &m, &g)?;

    for f in initial_frontier {
        heap.push(f);
    }

    loop {
        // todo: explore the leading frontier edge
        match heap.pop() {
            None => break,
            Some(f) => {
                // todo: transition frontier to a vertex-oriented model? review pseudocode
                //   - needs to apply Edge => Edge => Cost function
                //   - do we need to store travel time explicitly on the frontier to apply cost_estimate_fn?
                //   - should cost_estimate_fn come from the TraversalModel?
                //     - coupled with the State, we could know how to store/retrieve the relevant g score + apply the h score
                //     - h scores could then leverage same exact cost function easily
                // todo: evaluate h score (cost_estimate_fn) in expand
                // todo: finalize Solution model, assign results

                let expand_vertex_id = match direction {
                    Direction::Forward => g.dst_vertex(f.edge_id),
                    Direction::Reverse => g.src_vertex(f.edge_id),
                }
                .map_err(SearchError::GraphCorrectnessFailure)?;

                let terminate_for_user = m
                    .terminate_search(&f)
                    .map_err(SearchError::TraversalModelFailure)?;

                let terminate_for_target = target.map(|v| v == expand_vertex_id).unwrap_or(false);
                if terminate_for_user || terminate_for_target {
                    break;
                }

                let this_frontier =
                    expand(expand_vertex_id, f.prev_edge_id, direction, f.state, &m, &g)?;
                for next_f in this_frontier {
                    heap.push(next_f);
                }
            }
        }
    }

    return Result::Ok(im::hashmap![]);
}

fn expand<S: Eq + Clone + Copy>(
    vertex_id: VertexId,
    prev_edge_id: Option<EdgeId>,
    direction: Direction,
    prev_state: S,
    // traversal_model: Arc<RwLock<dyn TraversalModel<State = S>>>,
    // directed_graph: Arc<RwLock<dyn DirectedGraph>>,
    m: &RwLockReadGuard<dyn TraversalModel<State = S>>,
    g: &RwLockReadGuard<dyn DirectedGraph>,
) -> Result<im::Vector<Frontier<S>>, SearchError> {
    // find in or out edges from this vertex id
    let initial_edges = match direction {
        Direction::Forward => g.out_edges(vertex_id),
        Direction::Reverse => g.in_edges(vertex_id),
    }
    .map_err(SearchError::GraphCorrectnessFailure)?;

    let mut expanded: im::Vector<Frontier<S>> = im::vector![];

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

        let initial_frontier = Frontier {
            edge_id,
            prev_edge_id,
            state: traversal_state,
            cost: access_cost + traversal_cost,
        };

        expanded.push_back(initial_frontier);
    }

    return Ok(expanded);
}
