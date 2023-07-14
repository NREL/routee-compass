use super::access_result::AccessResult;
use super::function::function::{
    CostAggregationFunction, EdgeCostFunction, EdgeEdgeCostFunction, TerminateSearchFunction,
    ValidFrontierFunction,
};
use super::traversal_error::TraversalError;
use crate::model::traversal::state::search_state::SearchState;
use crate::model::traversal::state::state_variable::StateVar;
use crate::model::traversal::traversal_model_config::TraversalModelConfig;
use crate::model::traversal::traversal_result::TraversalResult;
use crate::{
    algorithm::search::min_search_tree::dijkstra::edge_frontier::EdgeFrontier,
    model::{
        cost::cost::Cost,
        property::{edge::Edge, vertex::Vertex},
    },
};

/// a TraversalModel holds the various rules for a search. during a typical
/// search:
/// - the search terminates if any TerminateSearchFunctions are "true"
/// - a frontier is only traversed if it is "true" for all ValidFrontierFunctions
/// - we apply all access cost functions, also referred to as EdgeEdgeCostFunctions,
///   when we attempt to traverse an edge
/// - we apply all traversal cost functions, also referred to as EdgeCostFunctions,
///   while traversing that edge
///
/// the state of the search is updated by each function. each cost function type
/// has it's own sub-vector within the search state. for example, if we have two
/// edge cost functions, first for "time", second for "energy", we would track the
/// state via
/// $ let state = vec![vec![0.0], vec![0.0]];
///
pub struct TraversalModel<'a> {
    edge_fns: Vec<&'a EdgeCostFunction>,
    edge_edge_fns: Vec<&'a EdgeEdgeCostFunction>,
    valid_fns: Vec<&'a ValidFrontierFunction>,
    terminate_fns: Vec<&'a TerminateSearchFunction>,
    edge_agg_fn: &'a CostAggregationFunction,
    edge_edge_agg_fn: &'a CostAggregationFunction,
    initial_state: SearchState,
    edge_edge_start_idx: usize,
}

impl<'a> From<&TraversalModelConfig<'a>> for TraversalModel<'a> {
    fn from(config: &TraversalModelConfig<'a>) -> Self {
        // pull out the cost functions from their configs
        let edge_fns = config
            .edge_fns
            .iter()
            .map(|c| c.cost_fn)
            .collect::<Vec<&EdgeCostFunction>>();
        let edge_edge_fns = config
            .edge_edge_fns
            .iter()
            .map(|c| c.cost_fn)
            .collect::<Vec<&EdgeEdgeCostFunction>>();

        // pull out any valid frontier functions
        let mut valid_fns = config
            .edge_fns
            .iter()
            .flat_map(|c| c.valid_fn)
            .collect::<Vec<&ValidFrontierFunction>>();
        valid_fns.extend(config.edge_edge_fns.iter().flat_map(|c| c.valid_fn));

        // pull out any terminate search functions
        let mut terminate_fns = config
            .edge_fns
            .iter()
            .flat_map(|c| c.terminate_fn)
            .collect::<Vec<&ValidFrontierFunction>>();
        terminate_fns.extend(config.edge_edge_fns.iter().flat_map(|c| c.terminate_fn));

        // compose the initial states for each cost function
        let mut initial_state = config
            .edge_fns
            .iter()
            .map(|c| c.init_state.clone())
            .collect::<SearchState>();
        initial_state.extend(config.edge_edge_fns.iter().map(|c| c.init_state.clone()));

        // count the index where Edge->Edge cost functions begin in the state index
        let edge_edge_start_idx = config.edge_fns.len();

        return TraversalModel {
            edge_fns,
            edge_edge_fns,
            valid_fns,
            terminate_fns,
            edge_agg_fn: config.edge_agg_fn,
            edge_edge_agg_fn: config.edge_edge_agg_fn,
            initial_state,
            edge_edge_start_idx,
        };
    }
}

impl<'a> TraversalModel<'a> {
    pub fn initial_state(&self) -> Vec<Vec<StateVar>> {
        self.initial_state.to_vec()
    }

    /// completes an edge traversal by applying all EdgeCostFunctions. the result
    /// is collected as a TraversalResult.
    pub fn traversal_cost(
        &self,
        src: &Vertex,
        edge: &Edge,
        dst: &Vertex,
        state: &Vec<Vec<StateVar>>,
    ) -> Result<TraversalResult, TraversalError> {
        let mut cost_vector: Vec<Cost> = vec![];
        let mut updated_state = state.to_vec();
        for (idx, func) in self.edge_fns.iter().enumerate() {
            let fn_state = updated_state
                .get(idx)
                .ok_or(TraversalError::InvalidStateVariableIndexError)?;
            let (result_cost, result_state) = func(src, edge, dst, fn_state)?;
            cost_vector.push(result_cost);
            updated_state[idx] = result_state;
        }
        let total_cost = (self.edge_agg_fn)(&cost_vector)?;
        let result = TraversalResult {
            total_cost,
            cost_vector,
            updated_state,
        };
        return Ok(result);
    }

    /// completes the edge access of the $dst edge by applying all EdgeEdgeCostFunctions. the result
    /// is collected as an AccessResult.
    pub fn access_cost(
        &self,
        v1: &Vertex,
        src: &Edge,
        v2: &Vertex,
        dst: &Edge,
        v3: &Vertex,
        state: &Vec<Vec<StateVar>>,
    ) -> Result<AccessResult, TraversalError> {
        let mut cost_vector: Vec<Cost> = vec![];
        let mut updated_state = state.to_vec();
        for (idx, func) in self.edge_edge_fns.iter().enumerate() {
            let ee_idx = self.edge_edge_start_idx + idx;
            let fn_state = updated_state
                .get(ee_idx)
                .ok_or(TraversalError::InvalidStateVariableIndexError)?;
            let (result_cost, result_state) = func(v1, src, v2, dst, v3, fn_state)?;
            cost_vector.push(result_cost);
            updated_state[idx] = result_state;
        }
        let total_cost = (self.edge_edge_agg_fn)(&cost_vector)?;
        let result = AccessResult {
            total_cost,
            cost_vector,
            updated_state,
        };
        return Ok(result);
    }

    /// if any valid_fn fails, we return false, otherwise, true
    /// base case: zero valid_fns -> returns true
    pub fn valid_frontier(&self, frontier: &EdgeFrontier) -> Result<bool, TraversalError> {
        for valid_fn in self.valid_fns.iter() {
            let is_valid = valid_fn(frontier)?;
            if !is_valid {
                return Ok(false);
            }
        }
        return Ok(true);
    }

    /// if any terminate_fn succeeds, we return true, otherwise, false
    /// base case: zero terminate_fns -> returns false
    pub fn terminate_search(&self, frontier: &EdgeFrontier) -> Result<bool, TraversalError> {
        for terminate_fn in self.terminate_fns.iter() {
            let terminate = terminate_fn(frontier)?;
            if terminate {
                return Ok(true);
            }
        }
        return Ok(false);
    }
}
