use super::network_utility_mapping::NetworkUtilityMapping;
use crate::model::utility::utility_error::UtilityError;
use crate::{
    model::utility::network::{
        network_access_utility_row::NetworkAccessUtilityRow,
        network_traversal_utility_row::NetworkTraversalUtilityRow,
    },
    util::fs::read_utils,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NetworkUtilityMappingBuilder {
    #[serde(rename = "traversal_lookup")]
    EdgeLookupBuilder { cost_input_file: String },
    #[serde(rename = "access_lookup")]
    EdgeEdgeLookupBuilder { cost_input_file: String },
    #[serde(rename = "combined")]
    Combined(Vec<NetworkUtilityMappingBuilder>),
}

impl NetworkUtilityMappingBuilder {
    pub fn build(&self) -> Result<NetworkUtilityMapping, UtilityError> {
        use NetworkUtilityMapping as NCM;
        use NetworkUtilityMappingBuilder as Builder;
        match self {
            Builder::EdgeLookupBuilder { cost_input_file } => {
                let lookup =
                    read_utils::from_csv::<NetworkTraversalUtilityRow>(cost_input_file, true, None)
                        .map_err(|source| UtilityError::CsvIoError { source })?
                        .iter()
                        .map(|row| (row.edge_id, row.cost))
                        .collect::<HashMap<_, _>>();
                Ok(NCM::EdgeLookup { lookup })
            }
            Builder::EdgeEdgeLookupBuilder { cost_input_file } => {
                let lookup =
                    read_utils::from_csv::<NetworkAccessUtilityRow>(cost_input_file, true, None)
                        .map_err(|source| UtilityError::CsvIoError { source })?
                        .iter()
                        .map(|row| ((row.source, row.destination), row.cost))
                        .collect::<HashMap<_, _>>();

                Ok(NCM::EdgeEdgeLookup { lookup })
            }
            Builder::Combined(builders) => {
                let mappings = builders
                    .iter()
                    .map(|b| b.build())
                    .collect::<Result<Vec<_>, UtilityError>>()?;
                Ok(NCM::Combined(mappings))
            }
        }
    }
}
