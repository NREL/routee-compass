use super::network_cost_mapping::NetworkCostMapping;
use crate::{
    model::cost_function::{
        cost_aggregation::CostAggregation,
        cost_mapping_error::CostMappingError,
        network::{
            network_access_cost_row::NetworkAccessCostRow,
            network_traversal_cost_row::NetworkTraversalCostRow,
        },
    },
    util::fs::read_utils,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub enum NetworkCostMappingBuilder {
    #[serde(rename = "traversal_lookup")]
    EdgeLookupBuilder { cost_input_file: String },
    #[serde(rename = "access_lookup")]
    EdgeEdgeLookupBuilder { cost_input_file: String },
    #[serde(rename = "combined")]
    Combined(Vec<NetworkCostMappingBuilder>, CostAggregation),
}

impl NetworkCostMappingBuilder {
    pub fn build(&self) -> Result<NetworkCostMapping, CostMappingError> {
        use NetworkCostMapping as NCM;
        use NetworkCostMappingBuilder as Builder;
        match self {
            Builder::EdgeLookupBuilder { cost_input_file } => {
                let lookup =
                    read_utils::from_csv::<NetworkTraversalCostRow>(cost_input_file, true, None)
                        .map_err(|source| CostMappingError::LookupFileIOError { source })?
                        .iter()
                        .map(|row| (row.edge_id, row.cost))
                        .collect::<HashMap<_, _>>();
                Ok(NCM::EdgeLookup { lookup })
            }
            Builder::EdgeEdgeLookupBuilder { cost_input_file } => {
                let lookup =
                    read_utils::from_csv::<NetworkAccessCostRow>(cost_input_file, true, None)
                        .map_err(|source| CostMappingError::LookupFileIOError { source })?
                        .iter()
                        .map(|row| ((row.source, row.destination), row.cost))
                        .collect::<HashMap<_, _>>();

                Ok(NCM::EdgeEdgeLookup { lookup })
            }
            Builder::Combined(builders, aggregate_op) => {
                let mappings = builders
                    .iter()
                    .map(|b| b.build())
                    .collect::<Result<Vec<_>, CostMappingError>>()?;
                Ok(NCM::Combined(mappings, *aggregate_op))
            }
        }
    }
}
