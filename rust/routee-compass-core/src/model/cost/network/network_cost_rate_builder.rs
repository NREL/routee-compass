use super::network_cost_rate::NetworkCostRate;
use crate::model::cost::CostModelError;
use crate::{
    model::cost::network::{
        network_access_cost_row::NetworkAccessUtilityRow,
        network_traversal_cost_row::NetworkTraversalUtilityRow,
    },
    util::fs::read_utils,
};
use kdam::Bar;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NetworkCostRateBuilder {
    #[serde(rename = "traversal_lookup")]
    EdgeLookupBuilder { cost_input_file: String },
    #[serde(rename = "access_lookup")]
    EdgeEdgeLookupBuilder { cost_input_file: String },
    #[serde(rename = "combined")]
    Combined(Vec<NetworkCostRateBuilder>),
}

impl NetworkCostRateBuilder {
    pub fn build(&self) -> Result<NetworkCostRate, CostModelError> {
        use NetworkCostRate as NCM;
        use NetworkCostRateBuilder as Builder;
        match self {
            Builder::EdgeLookupBuilder { cost_input_file } => {
                let lookup = read_utils::from_csv::<NetworkTraversalUtilityRow>(
                    cost_input_file,
                    true,
                    Some(Bar::builder().desc("network edge cost lookup")),
                    None,
                )
                .map_err(|source| {
                    CostModelError::BuildError(format!(
                        "failure reading file {}: {}",
                        cost_input_file, source
                    ))
                })?
                .iter()
                .map(|row| (row.edge_id, row.cost))
                .collect::<HashMap<_, _>>();
                Ok(NCM::EdgeLookup { lookup })
            }
            Builder::EdgeEdgeLookupBuilder { cost_input_file } => {
                let lookup = read_utils::from_csv::<NetworkAccessUtilityRow>(
                    cost_input_file,
                    true,
                    Some(Bar::builder().desc("network edge->edge cost lookup")),
                    None,
                )
                .map_err(|source| {
                    CostModelError::BuildError(format!(
                        "failure reading file {}: {}",
                        cost_input_file, source
                    ))
                })?
                .iter()
                .map(|row| ((row.source, row.destination), row.cost))
                .collect::<HashMap<_, _>>();

                Ok(NCM::EdgeEdgeLookup { lookup })
            }
            Builder::Combined(builders) => {
                let mappings = builders
                    .iter()
                    .map(|b| b.build())
                    .collect::<Result<Vec<_>, CostModelError>>()?;
                Ok(NCM::Combined(mappings))
            }
        }
    }
}
