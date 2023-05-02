use im;
use std::collections::BinaryHeap;

use crate::model::cost::cost::Cost;
use crate::model::cost::function::{CostEstFn, CostFn, EdgeEdgeMetricFn, EdgeMetricFn};
use crate::{
    algorithm::search::min_search_tree::{direction::Direction, frontier::Frontier},
    model::graph::{directed_graph::DirectedGraph, edge_id::EdgeId, vertex_id::VertexId},
};

use std::sync::{Arc, RwLock};

use super::solution::Solution;

type MinSearchTree = im::HashMap<EdgeId, Solution>;

pub fn run_search(
    g: Arc<RwLock<dyn DirectedGraph>>,
    direction: Direction,
    source: VertexId,
    target: Option<VertexId>,
    edge_metric_fn: Arc<RwLock<EdgeMetricFn>>,
    edge_edge_metric_fn: Arc<RwLock<EdgeEdgeMetricFn>>,
    cost_fn: Arc<RwLock<CostFn>>,
    cost_estimate_fn: Option<Arc<RwLock<CostEstFn>>>,
) -> Result<MinSearchTree, String> {
    if target.is_none() && cost_estimate_fn.is_none() {
        let msg = "distance heuristic can only be provided when there is a target";
        return Result::Err(msg.to_string());
    }

    // todo: setup

    let mut heap: BinaryHeap<Frontier> = BinaryHeap::new();
    let initial_edges = match direction {
        Direction::Forward => g.read().unwrap().out_edges(source),
        Direction::Reverse => g.read().unwrap().in_edges(source),
    };
    for edge_id in initial_edges? {
        let edge = g.read().unwrap().edge_attr(edge_id)?;

        // todo: sort out friction from having different error implementations
        // see https://kerkour.com/rust-error-handling
        // let edge_metric = edge_metric_fn.read().unwrap()(edge)?;

        // todo: both edges need to be in scope here to call this
        // but, we don't have a src edge/prev edge in scope yet!
        // let e_e_metric = edge_edge_metric_fn.read().unwrap()()

        let frontier = Frontier {
            edge_id,
            traverse_edge_metrics: im::vector![],
            edge_edge_metrics: im::vector![],
            cost: Cost::ZERO,
        };
        heap.push(frontier)
    }

    return Result::Ok(im::hashmap![]);
}
