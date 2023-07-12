use super::function::{
    edge_cost_function_config::EdgeCostFunctionConfig,
    edge_edge_cost_function_config::EdgeEdgeCostFunctionConfig,
};
use crate::model::traversal::function::function::CostAggregationFunction;

pub struct TraversalModelConfig<'a> {
    pub edge_fns: Vec<&'a EdgeCostFunctionConfig<'a>>,
    pub edge_edge_fns: Vec<&'a EdgeEdgeCostFunctionConfig<'a>>,
    pub edge_agg_fn: &'a CostAggregationFunction,
    pub edge_edge_agg_fn: &'a CostAggregationFunction,
}
