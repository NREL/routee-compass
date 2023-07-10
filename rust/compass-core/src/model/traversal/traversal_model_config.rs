use super::cost_function::{
    edge_cost_function_config::EdgeCostFunctionConfig,
    edge_edge_cost_function_config::EdgeEdgeCostFunctionConfig,
};

pub struct TraversalModelConfig<'a> {
    pub edge_fns: Vec<&'a EdgeCostFunctionConfig<'a>>,
    pub edge_edge_fns: Vec<&'a EdgeEdgeCostFunctionConfig<'a>>,
}
