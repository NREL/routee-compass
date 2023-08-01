use compass_core::{
    algorithm::search::min_search_tree::{
        a_star::{
            a_star::{backtrack, backtrack_edges, run_a_star, run_a_star_edge_oriented},
            cost_estimate_function::CostEstimateFunction,
        },
        direction::Direction,
    },
    model::{
        graph::{directed_graph::DirectedGraph, edge_id::EdgeId, vertex_id::VertexId},
        traversal::traversal_model::TraversalModel,
    },
    util::read_only_lock::DriverReadOnlyLock,
};
use rayon::prelude::*;
use std::sync::Arc;

use crate::app::app_error::AppError;

use super::search_app_result::SearchAppResult;

pub struct SearchApp<'app> {
    graph: Arc<DriverReadOnlyLock<&'app dyn DirectedGraph>>,
    a_star_heuristic: Arc<DriverReadOnlyLock<&'app dyn CostEstimateFunction>>,
    traversal_model: Arc<DriverReadOnlyLock<&'app TraversalModel>>,
}

impl<'app> SearchApp<'app> {
    /// builds a new CompassApp from the required components.
    /// handles all of the specialized boxing that allows for simple parallelization.
    pub fn new(
        graph: &'app dyn DirectedGraph,
        traversal_model: &'app TraversalModel,
        a_star_heuristic: &'app dyn CostEstimateFunction,
    ) -> Self {
        let g = Arc::new(DriverReadOnlyLock::new(graph as &dyn DirectedGraph));
        let h = Arc::new(DriverReadOnlyLock::new(
            a_star_heuristic as &dyn CostEstimateFunction,
        ));
        let t = Arc::new(DriverReadOnlyLock::new(traversal_model));
        return SearchApp {
            graph: g,
            a_star_heuristic: h,
            traversal_model: t,
        };
    }

    ///
    /// runs a set of queries in parallel against the state of this CompassApp
    ///
    pub fn run_vertex_oriented(
        &self,
        queries: Vec<(VertexId, VertexId)>,
    ) -> Result<Vec<SearchAppResult<VertexId>>, AppError> {
        // execute the route search
        let result: Vec<Result<SearchAppResult<VertexId>, AppError>> = queries
            .clone()
            .into_par_iter()
            .map(|(o, d)| {
                let dg_inner = Arc::new(self.graph.read_only());
                let tm_inner = Arc::new(self.traversal_model.read_only());
                let cost_inner = Arc::new(self.a_star_heuristic.read_only());
                run_a_star(Direction::Forward, o, d, dg_inner, tm_inner, cost_inner)
                    .and_then(|tree| backtrack(o, d, tree))
                    .and_then(|route| {
                        Ok(SearchAppResult {
                            origin: o,
                            destination: d,
                            route,
                        })
                    })
                    .map_err(AppError::SearchError)
            })
            .collect();

        return result.into_iter().collect();
    }

    ///
    /// runs a set of queries in parallel against the state of this CompassApp
    ///
    pub fn run_edge_oriented(
        &self,
        queries: Vec<(EdgeId, EdgeId)>,
    ) -> Result<Vec<SearchAppResult<EdgeId>>, AppError> {
        // execute the route search
        let result: Vec<Result<SearchAppResult<EdgeId>, AppError>> = queries
            .clone()
            .into_par_iter()
            .map(|(o, d)| {
                let dg_inner_search = Arc::new(self.graph.read_only());
                let dg_inner_backtrack = Arc::new(self.graph.read_only());
                let tm_inner = Arc::new(self.traversal_model.read_only());
                let cost_inner = Arc::new(self.a_star_heuristic.read_only());
                run_a_star_edge_oriented(
                    Direction::Forward,
                    o,
                    d,
                    dg_inner_search,
                    tm_inner,
                    cost_inner,
                )
                .and_then(|tree| backtrack_edges(o, d, tree, dg_inner_backtrack))
                .and_then(|route| {
                    Ok(SearchAppResult {
                        origin: o,
                        destination: d,
                        route,
                    })
                })
                .map_err(AppError::SearchError)
            })
            .collect();

        return result.into_iter().collect();
    }
}
