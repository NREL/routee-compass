use compass_core::model::traversal::{
    function::{
        default::{
            aggregation::{additive_aggregation, multiplicitive_aggregation},
            distance_cost::{distance_cost_function, initial_distance_state},
        },
        function::{
            CostAggregationFunction, EdgeCostFunction, EdgeEdgeCostFunction, ValidFrontierFunction,
        },
    },
    state::search_state::{SearchState, StateVector},
    traversal_model::TraversalModel,
};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub enum AlgorithmConfig {
    #[serde(rename = "astar")]
    AStar {
        heuristic: String,
        bidirectional: bool,
    },
}

#[derive(Debug, Deserialize, Clone)]
pub enum AggregationFunctionConfig {
    #[serde(rename = "add")]
    Add,
    #[serde(rename = "multiply")]
    Multiply,
}

impl AggregationFunctionConfig {
    pub fn into_aggregation_function(&self) -> CostAggregationFunction {
        match self {
            AggregationFunctionConfig::Add => additive_aggregation(),
            AggregationFunctionConfig::Multiply => multiplicitive_aggregation(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum EdgeCostFunctionConfig {
    #[serde(rename = "distance")]
    Distance,
    #[serde(rename = "free_flow")]
    FreeFlow,
    #[serde(rename = "powertrain")]
    Powertrain { model: String },
}

impl EdgeCostFunctionConfig {
    pub fn into_cost_function(&self) -> EdgeCostFunction {
        match self {
            EdgeCostFunctionConfig::Distance => distance_cost_function(),
            EdgeCostFunctionConfig::FreeFlow => panic!("FreeFlow cost function not implemented"),
            EdgeCostFunctionConfig::Powertrain { model } => {
                panic!("Powertrain cost function not implemented")
            }
        }
    }
    pub fn into_initial_state(&self) -> StateVector {
        match self {
            EdgeCostFunctionConfig::Distance => initial_distance_state(),
            EdgeCostFunctionConfig::FreeFlow => panic!("FreeFlow cost function not implemented"),
            EdgeCostFunctionConfig::Powertrain { model } => {
                panic!("Powertrain cost function not implemented")
            }
        }
    }
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(tag = "type")]
pub enum EdgeEdgeCostFunctionConfig {}

impl EdgeEdgeCostFunctionConfig {
    pub fn into_cost_function(&self) -> EdgeEdgeCostFunction {
        panic!("No edge edge cost function implemented yet")
    }

    pub fn into_initial_state(&self) -> StateVector {
        panic!("No edge edge cost function implemented yet")
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum ValidFrontierFunctionConfig {}

impl ValidFrontierFunctionConfig {
    pub fn into_valid_function(&self) -> ValidFrontierFunction {
        panic!("No valid frontier function implemented yet")
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum TerminateSearchFunctionConfig {}

impl TerminateSearchFunctionConfig {
    pub fn into_terminate_function(&self) -> ValidFrontierFunction {
        panic!("No terminate search function implemented yet")
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct TraversalModelConfig {
    edge_cost_functions: Vec<EdgeCostFunctionConfig>,
    edge_edge_cost_functions: Vec<EdgeEdgeCostFunctionConfig>,
    valid_functions: Vec<ValidFrontierFunctionConfig>,
    terminate_functions: Vec<TerminateSearchFunctionConfig>,
    edge_aggregation_function: AggregationFunctionConfig,
    edge_edge_aggregation_function: AggregationFunctionConfig,
}

impl TraversalModelConfig {
    pub fn into_traversal_model(&self) -> TraversalModel {
        let edge_fns = self
            .edge_cost_functions
            .iter()
            .map(|c| c.into_cost_function())
            .collect::<Vec<EdgeCostFunction>>();
        let edge_edge_fns = self
            .edge_edge_cost_functions
            .iter()
            .map(|c| c.into_cost_function())
            .collect::<Vec<EdgeEdgeCostFunction>>();
        let valid_fns = self
            .valid_functions
            .iter()
            .map(|c| c.into_valid_function())
            .collect::<Vec<ValidFrontierFunction>>();
        let terminate_fns = self
            .terminate_functions
            .iter()
            .map(|c| c.into_terminate_function())
            .collect::<Vec<ValidFrontierFunction>>();
        let mut initial_state = self
            .edge_cost_functions
            .iter()
            .map(|c| c.into_initial_state())
            .collect::<SearchState>();

        initial_state.extend(
            self.edge_edge_cost_functions
                .iter()
                .map(|c| c.into_initial_state()),
        );

        let edge_edge_start_idx = self.edge_cost_functions.len();

        let traversal_model = TraversalModel {
            edge_fns,
            edge_edge_fns,
            valid_fns,
            terminate_fns,
            edge_agg_fn: self
                .edge_aggregation_function
                .into_aggregation_function(),
            edge_edge_agg_fn: self
                .edge_edge_aggregation_function
                .into_aggregation_function(),
            initial_state,
            edge_edge_start_idx,
        };

        traversal_model
    }
}

#[derive(Debug, Deserialize)]
pub struct SearchConfig {
    pub algorithm: AlgorithmConfig,
    pub traversal_model: TraversalModelConfig,
}
