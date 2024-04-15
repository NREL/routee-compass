use super::{search_app_ops, search_app_result::SearchAppResult};
use crate::{
    app::compass::{
        compass_app_error::CompassAppError,
        compass_input_field::CompassInputField,
        config::{
            compass_configuration_field::CompassConfigurationField,
            cost_model::cost_model_service::CostModelService,
        },
        search_orientation::{self, SearchOrientation},
    },
    plugin::input::{input_field::InputField, input_json_extensions::InputJsonExtensions},
};
use chrono::Local;
use itertools::Itertools;
use routee_compass_core::{
    algorithm::search::{
        backtrack, edge_traversal::EdgeTraversal, search_algorithm::SearchAlgorithm,
        search_error::SearchError, search_instance::SearchInstance,
        search_tree_branch::SearchTreeBranch,
    },
    model::{
        access::access_model_service::AccessModelService,
        frontier::frontier_model_service::FrontierModelService, road_network::graph::Graph,
        state::state_model::StateModel, termination::termination_model::TerminationModel,
        traversal::traversal_model_service::TraversalModelService, unit::Cost,
    },
};
use serde_json::json;
use std::sync::Arc;
use std::time;

/// a configured and loaded application to execute searches.
pub struct SearchApp {
    pub search_algorithm: SearchAlgorithm,
    pub directed_graph: Arc<Graph>,
    pub state_model: Arc<StateModel>,
    pub traversal_model_service: Arc<dyn TraversalModelService>,
    pub access_model_service: Arc<dyn AccessModelService>,
    pub cost_model_service: Arc<CostModelService>,
    pub frontier_model_service: Arc<dyn FrontierModelService>,
    pub termination_model: Arc<TerminationModel>,
}

impl SearchApp {
    /// builds a new SearchApp from the required components.
    /// handles all of the specialized boxing that allows for simple parallelization.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        search_algorithm: SearchAlgorithm,
        graph: Graph,
        state_model: Arc<StateModel>,
        traversal_model_service: Arc<dyn TraversalModelService>,
        access_model_service: Arc<dyn AccessModelService>,
        cost_model_service: CostModelService,
        frontier_model_service: Arc<dyn FrontierModelService>,
        termination_model: TerminationModel,
    ) -> Self {
        SearchApp {
            search_algorithm,
            directed_graph: Arc::new(graph),
            state_model,
            traversal_model_service,
            access_model_service,
            cost_model_service: Arc::new(cost_model_service),
            frontier_model_service,
            termination_model: Arc::new(termination_model),
        }
    }

    pub fn run(
        &self,
        query: &serde_json::Value,
        search_orientation: &SearchOrientation,
    ) -> Result<(SearchAppResult, SearchInstance), CompassAppError> {
        match search_orientation {
            SearchOrientation::Vertex => self.run_vertex_oriented(query),
            SearchOrientation::Edge => self.run_edge_oriented(query),
        }
    }

    /// runs a single vertex oriented query
    ///
    pub fn run_vertex_oriented(
        &self,
        query: &serde_json::Value,
    ) -> Result<(SearchAppResult, SearchInstance), CompassAppError> {
        let o = query
            .get_origin_vertex()
            .map_err(CompassAppError::PluginError)?;
        let d = query
            .get_destination_vertex()
            .map_err(CompassAppError::PluginError)?;
        let search_start_time = Local::now();

        let search_instance = self.build_search_instance(query)?;
        self.search_algorithm
            .run_vertex_oriented(o, d, &search_instance)
            .and_then(|search_result| {
                let search_end_time = Local::now();
                let search_runtime = (search_end_time - search_start_time)
                    .to_std()
                    .unwrap_or(time::Duration::ZERO);
                log::debug!(
                    "Search Completed in {:?} miliseconds",
                    search_runtime.as_millis()
                );
                let route_start_time = Local::now();
                let route = match d {
                    None => vec![],
                    Some(dest) => backtrack::vertex_oriented_route(o, dest, &search_result.tree)?,
                };
                let route_end_time = Local::now();
                let route_runtime = (route_end_time - route_start_time)
                    .to_std()
                    .unwrap_or(time::Duration::ZERO);
                log::debug!(
                    "Route Computed in {:?} miliseconds",
                    route_runtime.as_millis()
                );
                let result = SearchAppResult {
                    routes: vec![route],
                    trees: vec![search_result.tree],
                    search_executed_time: search_start_time.to_rfc3339(),
                    algorithm_runtime: search_runtime,
                    route_runtime,
                    search_app_runtime: search_runtime + route_runtime,
                    iterations: search_result.iterations,
                };
                Ok((result, search_instance))
            })
            .map_err(CompassAppError::SearchError)
    }

    pub fn run_edge_oriented(
        &self,
        query: &serde_json::Value,
    ) -> Result<(SearchAppResult, SearchInstance), CompassAppError> {
        let search_start_time = Local::now();
        let o = query
            .get_origin_edge()
            .map_err(CompassAppError::PluginError)?;
        let d_opt = query
            .get_destination_edge()
            .map_err(CompassAppError::PluginError)?;
        // let search_instance = self.build_search_instance(query)?;

        // prepare the vertex-oriented JSON query
        let mut v_query = query.clone();
        let e1dv_id = self.directed_graph.dst_vertex_id(o)?;
        let e1dv = self.directed_graph.get_vertex(e1dv_id)?;
        v_query[InputField::OriginX.to_str()] = json![e1dv.x()];
        v_query[InputField::OriginY.to_str()] = json![e1dv.y()];

        if let Some(d_id) = d_opt {
            let e2sv_id = self.directed_graph.src_vertex_id(d_id)?;
            let e2sv = self.directed_graph.get_vertex(e2sv_id)?;
            v_query[InputField::DestinationX.to_str()] = json![e2sv.x()];
            v_query[InputField::DestinationY.to_str()] = json![e2sv.y()];
        }

        let (v_result, si) = self.run_vertex_oriented(&v_query)?;

        // prepend a zero-cost dummy link to the routes
        let src_et = EdgeTraversal {
            edge_id: o,
            access_cost: Cost::ZERO,
            traversal_cost: Cost::ZERO,
            result_state: si.state_model.initial_state()?,
        };
        let routes_w_src_edge = v_result
            .routes
            .into_iter()
            .map(|r| {
                r.insert(0, src_et.clone());
                r
            })
            .collect_vec();

        let routes_w_dst_edge = if let Some(d_id) = d_opt {
            v_result
                .routes
                .into_iter()
                .map(|r| match r.last() {
                    None => r,
                    Some(last_edge) => {
                        let dst_et = EdgeTraversal {
                            edge_id: d_id,
                            access_cost: Cost::ZERO,
                            traversal_cost: Cost::ZERO,
                            result_state: last_edge.result_state.clone(),
                        };
                        r.push(dst_et);
                        r
                    }
                })
                .collect_vec()
        } else {
            routes_w_src_edge
        };

        let updated_result = SearchAppResult {
            routes: routes_w_dst_edge,
            trees: v_result.trees,
            search_executed_time: v_result.search_executed_time,
            algorithm_runtime: v_result.algorithm_runtime,
            route_runtime: v_result.route_runtime,
            search_app_runtime: v_result.search_app_runtime,
            iterations: v_result.iterations,
        };
        return Ok((updated_result, si));
    }
    // // 1. guard against edge conditions (src==dst, src.dst_v == dst.src_v)
    // let e1_src = self.directed_graph.src_vertex_id(source)?;
    // // let e1_dst = self.directed_graph.dst_vertex_id(source)?;
    // let src_et = EdgeTraversal {
    //     edge_id: source,
    //     access_cost: Cost::ZERO,
    //     traversal_cost: Cost::ZERO,
    //     result_state: si.state_model.initial_state()?,
    // };
    // let src_branch = SearchTreeBranch {
    //     terminal_vertex: e1_src,
    //     edge_traversal: src_et,
    // };

    // match target {
    //     None => {
    //         // let
    //         let SearchResult {
    //             mut tree,
    //             iterations,
    //         } = run_vertex_oriented(e1_dst, None, si)?;
    //         if !tree.contains_key(&e1_dst) {
    //             tree.extend([(e1_dst, src_branch)]);
    //         }
    //         let updated = SearchResult {
    //             tree,
    //             iterations: iterations + 1,
    //         };
    //         Ok(updated)
    //     }
    //     Some(target_edge) => {
    //         let e2_src = si.directed_graph.src_vertex_id(target_edge)?;
    //         let e2_dst = si.directed_graph.dst_vertex_id(target_edge)?;

    //         if source == target_edge {
    //             Ok(SearchResult::default())
    //         } else if e1_dst == e2_src {
    //             // route is simply source -> target
    //             let init_state = si.state_model.initial_state()?;
    //             let src_et = EdgeTraversal::perform_traversal(source, None, &init_state, si)?;
    //             let dst_et = EdgeTraversal::perform_traversal(
    //                 target_edge,
    //                 Some(source),
    //                 &src_et.result_state,
    //                 si,
    //             )?;
    //             let src_traversal = SearchTreeBranch {
    //                 terminal_vertex: e2_src,
    //                 edge_traversal: dst_et,
    //             };
    //             let dst_traversal = SearchTreeBranch {
    //                 terminal_vertex: e1_src,
    //                 edge_traversal: src_et,
    //             };
    //             let tree = HashMap::from([(e2_dst, src_traversal), (e1_dst, dst_traversal)]);
    //             let result = SearchResult {
    //                 tree,
    //                 iterations: 1,
    //             };
    //             return Ok(result);
    //         } else {
    //             // run a search and append source/target edges to result
    //             let SearchResult {
    //                 mut tree,
    //                 iterations,
    //             } = run_a_star(e1_dst, Some(e2_src), si)?;

    //             if tree.is_empty() {
    //                 return Err(SearchError::NoPathExists(e1_dst, e2_src));
    //             }

    //             let final_state = &tree
    //                 .get(&e2_src)
    //                 .ok_or_else(|| SearchError::VertexMissingFromSearchTree(e2_src))?
    //                 .edge_traversal
    //                 .result_state;
    //             let dst_et = EdgeTraversal {
    //                 edge_id: target_edge,
    //                 access_cost: Cost::ZERO,
    //                 traversal_cost: Cost::ZERO,
    //                 result_state: final_state.to_vec(),
    //             };
    //             let dst_traversal = SearchTreeBranch {
    //                 terminal_vertex: e2_src,
    //                 edge_traversal: dst_et,
    //             };

    //             // it is possible that the search already found these vertices. one major edge
    //             // case is when the trip starts with a u-turn.
    //             if !tree.contains_key(&e1_dst) {
    //                 tree.extend([(e1_dst, src_branch)]);
    //             }
    //             if !tree.contains_key(&e2_dst) {
    //                 tree.extend([(e2_dst, dst_traversal)]);
    //             }

    //             let result = SearchResult {
    //                 tree,
    //                 iterations: iterations + 2,
    //             };
    //             Ok(result)
    //         }
    //     }
    // }

    // ///
    // /// runs a single edge oriented query
    // ///
    // pub fn run_edge_oriented(
    //     &self,
    //     query: &serde_json::Value,
    // ) -> Result<(SearchAppResult, SearchInstance), CompassAppError> {
    //     let o = query
    //         .get_origin_edge()
    //         .map_err(CompassAppError::PluginError)?;
    //     let d = query
    //         .get_destination_edge()
    //         .map_err(CompassAppError::PluginError)?;
    //     let search_start_time = Local::now();
    //     let search_instance = self.build_search_instance(query)?;
    //     self.search_algorithm
    //         .run_edge_oriented(o, d, &search_instance)
    //         .and_then(|search_result| {
    //             let search_end_time = Local::now();
    //             let route_start_time = Local::now();
    //             let route = match d {
    //                 None => vec![],
    //                 Some(dest) => backtrack::edge_oriented_route(
    //                     o,
    //                     dest,
    //                     &search_result.tree,
    //                     search_instance.directed_graph.clone(),
    //                 )?,
    //             };
    //             let route_end_time = Local::now();
    //             let search_runtime = (search_end_time - search_start_time)
    //                 .to_std()
    //                 .unwrap_or(time::Duration::ZERO);
    //             let route_runtime = (route_end_time - route_start_time)
    //                 .to_std()
    //                 .unwrap_or(time::Duration::ZERO);
    //             let result = SearchAppResult {
    //                 routes: vec![route],
    //                 trees: vec![search_result.tree],
    //                 search_executed_time: search_start_time.to_rfc3339(),
    //                 algorithm_runtime: search_runtime,
    //                 route_runtime,
    //                 search_app_runtime: search_runtime + route_runtime,
    //                 iterations: search_result.iterations,
    //             };
    //             Ok((result, search_instance))
    //         })
    //         .map_err(CompassAppError::SearchError)
    // }

    pub fn build_search_instance(
        &self,
        query: &serde_json::Value,
    ) -> Result<SearchInstance, SearchError> {
        let traversal_model = self.traversal_model_service.build(query)?;
        let access_model = self.access_model_service.build(query)?;

        let state_features =
            search_app_ops::collect_features(query, traversal_model.clone(), access_model.clone())?;
        let state_model_instance = self.state_model.extend(state_features)?;
        let state_model = Arc::new(state_model_instance);

        let cost_model = self
            .cost_model_service
            .build(query, state_model.clone())
            .map_err(|e| SearchError::BuildError(e.to_string()))?;
        let frontier_model = self
            .frontier_model_service
            .build(query, state_model.clone())?;

        let search_assets = SearchInstance {
            directed_graph: self.directed_graph.clone(),
            state_model,
            traversal_model,
            access_model,
            cost_model,
            frontier_model,
            termination_model: self.termination_model.clone(),
        };

        Ok(search_assets)
    }
}
