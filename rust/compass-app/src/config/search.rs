use compass_core::model::traversal::function::default::velocity::edge_velocity_lookup::{
    build_edge_velocity_lookup, initial_velocity_state,
};
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
use compass_core::model::units::TimeUnit;
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
    type Error = String;

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
    #[serde(rename = "velocity_table")]
    VelocityTable {
        filename: String,
        output_unit: TimeUnit,
    },
    #[serde(rename = "powertrain")]
    Powertrain { model: String },
}

impl TryFrom<&EdgeCostFunctionConfig> for EdgeCostFunction {
    type Error = String;

    fn try_from(value: &EdgeCostFunctionConfig) -> Result<Self, Self::Error> {
        match value {
            EdgeCostFunctionConfig::Distance => Ok(distance_cost_function()),
            EdgeCostFunctionConfig::VelocityTable {
                filename,
                output_unit,
            } => {
                build_edge_velocity_lookup(filename, output_unit.clone()).map_err(|e| e.to_string())
            }
            EdgeCostFunctionConfig::Powertrain { model } => {
                Err(String::from("Powertrain cost function not implemented"))
            }
        }
    }
}

impl TryFrom<&EdgeCostFunctionConfig> for StateVector {
    type Error = String;

    fn try_from(value: &EdgeCostFunctionConfig) -> Result<Self, Self::Error> {
        match value {
            EdgeCostFunctionConfig::Distance => Ok(initial_distance_state()),
            EdgeCostFunctionConfig::VelocityTable {
                filename,
                output_unit,
            } => Ok(initial_velocity_state()),
            EdgeCostFunctionConfig::Powertrain { model } => {
                Err(String::from("Powertrain cost function not implemented"))
            }
        }
    }
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(tag = "type")]
pub enum EdgeEdgeCostFunctionConfig {}

impl TryFrom<&EdgeEdgeCostFunctionConfig> for EdgeEdgeCostFunction {
    type Error = String;

    fn try_from(value: &EdgeEdgeCostFunctionConfig) -> Result<Self, Self::Error> {
        Err(String::from("No edge edge cost function implemented yet"))
    }
}

impl TryFrom<&EdgeEdgeCostFunctionConfig> for StateVector {
    type Error = String;

    fn try_from(value: &EdgeEdgeCostFunctionConfig) -> Result<Self, Self::Error> {
        Err(String::from("No edge edge initial state implemented yet"))
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum ValidFrontierFunctionConfig {}

impl TryFrom<&ValidFrontierFunctionConfig> for ValidFrontierFunction {
    type Error = String;

    fn try_from(value: &ValidFrontierFunctionConfig) -> Result<Self, Self::Error> {
        Err(String::from("No valid frontier function implemented yet"))
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum TerminateSearchFunctionConfig {}

impl TryFrom<&TerminateSearchFunctionConfig> for TerminateSearchFunction {
    type Error = String;

    fn try_from(value: &TerminateSearchFunctionConfig) -> Result<Self, Self::Error> {
        Err(String::from("No terminate search function implemented yet"))
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
    type Error = String;

    fn try_from(value: TraversalModelConfig) -> Result<TraversalModel, Self::Error> {
        let edge_fns = value
            .edge_cost_functions
            .iter()
            .map(EdgeCostFunction::try_from)
            .collect::<Result<Vec<EdgeCostFunction>, String>>()?;
        let edge_edge_fns = value
            .edge_edge_cost_functions
            .iter()
            .map(EdgeEdgeCostFunction::try_from)
            .collect::<Result<Vec<EdgeEdgeCostFunction>, String>>()?;
        let valid_fns = value
            .valid_functions
            .iter()
            .map(ValidFrontierFunction::try_from)
            .collect::<Result<Vec<ValidFrontierFunction>, String>>()?;
        let terminate_fns = value
            .terminate_functions
            .iter()
            .map(TerminateSearchFunction::try_from)
            .collect::<Result<Vec<TerminateSearchFunction>, String>>()?;
        let mut initial_state = value
            .edge_cost_functions
            .iter()
            .map(StateVector::try_from)
            .collect::<Result<SearchState, String>>()?;

        initial_state.extend(
            value
                .edge_edge_cost_functions
                .iter()
                .map(StateVector::try_from)
                .collect::<Result<SearchState, String>>()?,
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
