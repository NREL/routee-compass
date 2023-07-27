use compass_core::model::traversal::{
    function::{
        default::{
            aggregation::{additive_aggregation, multiplicitive_aggregation},
            distance_cost::{distance_cost_function, initial_distance_state},
        },
        function::{
            CostAggregationFunction, EdgeCostFunction, EdgeEdgeCostFunction,
            TerminateSearchFunction, ValidFrontierFunction,
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

impl TryFrom<&AggregationFunctionConfig> for CostAggregationFunction {
    type Error = &'static str;

    fn try_from(value: &AggregationFunctionConfig) -> Result<Self, Self::Error> {
        match value {
            AggregationFunctionConfig::Add => Ok(additive_aggregation()),
            AggregationFunctionConfig::Multiply => Ok(multiplicitive_aggregation()),
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

impl TryFrom<&EdgeCostFunctionConfig> for EdgeCostFunction {
    type Error = &'static str;

    fn try_from(value: &EdgeCostFunctionConfig) -> Result<Self, Self::Error> {
        match value {
            EdgeCostFunctionConfig::Distance => Ok(distance_cost_function()),
            EdgeCostFunctionConfig::FreeFlow => Err("FreeFlow cost function not implemented"),
            EdgeCostFunctionConfig::Powertrain { model } => {
                Err("Powertrain cost function not implemented")
            }
        }
    }
}

impl TryFrom<&EdgeCostFunctionConfig> for StateVector {
    type Error = &'static str;

    fn try_from(value: &EdgeCostFunctionConfig) -> Result<Self, Self::Error> {
        match value {
            EdgeCostFunctionConfig::Distance => Ok(initial_distance_state()),
            EdgeCostFunctionConfig::FreeFlow => Err("FreeFlow cost function not implemented"),
            EdgeCostFunctionConfig::Powertrain { model } => {
                Err("Powertrain cost function not implemented")
            }
        }
    }
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(tag = "type")]
pub enum EdgeEdgeCostFunctionConfig {}

impl TryFrom<&EdgeEdgeCostFunctionConfig> for EdgeEdgeCostFunction {
    type Error = &'static str;

    fn try_from(value: &EdgeEdgeCostFunctionConfig) -> Result<Self, Self::Error> {
        Err("No edge edge cost function implemented yet")
    }
}

impl TryFrom<&EdgeEdgeCostFunctionConfig> for StateVector {
    type Error = &'static str;

    fn try_from(value: &EdgeEdgeCostFunctionConfig) -> Result<Self, Self::Error> {
        Err("No edge edge initial state implemented yet")
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum ValidFrontierFunctionConfig {}

impl TryFrom<&ValidFrontierFunctionConfig> for ValidFrontierFunction {
    type Error = &'static str;

    fn try_from(value: &ValidFrontierFunctionConfig) -> Result<Self, Self::Error> {
        Err("No valid frontier function implemented yet")
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum TerminateSearchFunctionConfig {}

impl TryFrom<&TerminateSearchFunctionConfig> for TerminateSearchFunction {
    type Error = &'static str;

    fn try_from(value: &TerminateSearchFunctionConfig) -> Result<Self, Self::Error> {
        Err("No terminate search function implemented yet")
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

impl TryFrom<TraversalModelConfig> for TraversalModel {
    type Error = &'static str;

    fn try_from(value: TraversalModelConfig) -> Result<TraversalModel, Self::Error> {
        let edge_fns = value
            .edge_cost_functions
            .iter()
            .map(EdgeCostFunction::try_from)
            .collect::<Result<Vec<EdgeCostFunction>, &'static str>>()?;
        let edge_edge_fns = value
            .edge_edge_cost_functions
            .iter()
            .map(EdgeEdgeCostFunction::try_from)
            .collect::<Result<Vec<EdgeEdgeCostFunction>, &'static str>>()?;
        let valid_fns = value
            .valid_functions
            .iter()
            .map(ValidFrontierFunction::try_from)
            .collect::<Result<Vec<ValidFrontierFunction>, &'static str>>()?;
        let terminate_fns = value
            .terminate_functions
            .iter()
            .map(TerminateSearchFunction::try_from)
            .collect::<Result<Vec<TerminateSearchFunction>, &'static str>>()?;
        let mut initial_state = value
            .edge_cost_functions
            .iter()
            .map(StateVector::try_from)
            .collect::<Result<SearchState, &'static str>>()?;

        initial_state.extend(
            value
                .edge_edge_cost_functions
                .iter()
                .map(StateVector::try_from)
                .collect::<Result<SearchState, &'static str>>()?,
        );

        let edge_agg_fn = CostAggregationFunction::try_from(&value.edge_aggregation_function)?;
        let edge_edge_agg_fn =
            CostAggregationFunction::try_from(&value.edge_edge_aggregation_function)?;

        let edge_edge_start_idx = value.edge_cost_functions.len();

        let traversal_model = TraversalModel {
            edge_fns,
            edge_edge_fns,
            valid_fns,
            terminate_fns,
            edge_agg_fn,
            edge_edge_agg_fn,
            initial_state,
            edge_edge_start_idx,
        };

        Ok(traversal_model)
    }
}

#[derive(Debug, Deserialize)]
pub struct SearchConfig {
    pub algorithm: AlgorithmConfig,
    pub traversal_model: TraversalModelConfig,
}
