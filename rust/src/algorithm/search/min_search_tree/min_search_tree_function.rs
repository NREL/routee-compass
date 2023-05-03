use std::collections::{BinaryHeap, HashMap};

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

type MinSearchTree = im::HashMap<EdgeId, Solution>;

pub fn run_search<S: Eq>(
    directed_graph: Arc<RwLock<dyn DirectedGraph>>,
    direction: Direction,
    source: VertexId,
    target: Option<VertexId>,
    traversal_model: Arc<RwLock<dyn TraversalModel<State = S>>>,
    // edge_metric_fn: Arc<RwLock<EdgeMetricFn>>,
    // edge_edge_metric_fn: Arc<RwLock<EdgeEdgeMetricFn>>,
    // cost_fn: Arc<RwLock<CostFn>>,
    cost_estimate_fn: Option<Arc<RwLock<CostEstFn>>>,
) -> Result<MinSearchTree, SearchError> {
    if target.is_none() && cost_estimate_fn.is_none() {
        return Result::Err(SearchError::DistanceHeuristicWithNoTarget);
    }

    // context for the search (graph, search functions, frontier priority queue)
    let g = directed_graph.read().unwrap();
    let m = traversal_model.read().unwrap();
    let mut heap: BinaryHeap<Frontier<S>> = BinaryHeap::new();
    let mut tree: HashMap<VertexId, Solution> = HashMap::new();

    // set up the initial search frontier
    let initial_edges = match direction {
        Direction::Forward => g.out_edges(source),
        Direction::Reverse => g.in_edges(source),
    }
    .map_err(SearchError::GraphCorrectnessFailure)?;
    for edge_id in initial_edges {
        let edge = g
            .edge_attr(edge_id)
            .map_err(SearchError::GraphCorrectnessFailure)?;

        let initial_state = m
            .initial_state(edge)
            .map_err(SearchError::TraversalModelFailure)?;

        let frontier = Frontier {
            edge_id,
            state: initial_state,
            cost: Cost::ZERO,
        };
        heap.push(frontier)
    }

    // todo: termination condition
    loop {
        // todo: explore the leading frontier edge
    }

    return Result::Ok(im::hashmap![]);
}
