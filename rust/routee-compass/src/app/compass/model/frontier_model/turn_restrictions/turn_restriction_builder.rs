use super::turn_restriction_service::{RestrictedEdgePair, TurnRestrictionFrontierService};
use kdam::Bar;
use routee_compass_core::config::{CompassConfigurationField, ConfigJsonExtensions};
use routee_compass_core::{
    model::frontier::{FrontierModelBuilder, FrontierModelError, FrontierModelService},
    util::fs::read_utils,
};
use std::{collections::HashSet, sync::Arc};

pub struct TurnRestrictionBuilder {}

impl FrontierModelBuilder for TurnRestrictionBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn FrontierModelService>, FrontierModelError> {
        let frontier_key = CompassConfigurationField::Frontier.to_string();
        let turn_restriction_file_key = String::from("turn_restriction_input_file");

        let turn_restriction_file = parameters
            .get_config_path(&turn_restriction_file_key, &frontier_key)
            .map_err(|e| {
                FrontierModelError::BuildError(format!(
                    "configuration error due to {}: {}",
                    turn_restriction_file_key.clone(),
                    e
                ))
            })?;

        let restricted_edges: HashSet<RestrictedEdgePair> = read_utils::from_csv(
            &turn_restriction_file,
            true,
            Some(Bar::builder().desc("turn restrictions")),
            None,
        )
        .map_err(|e| {
            FrontierModelError::BuildError(format!(
                "configuration error due to {}: {}",
                turn_restriction_file_key.clone(),
                e
            ))
        })?
        .iter()
        .cloned()
        .collect();

        log::debug!(
            "Loaded {} turn restrictions from {:?}.",
            restricted_edges.len(),
            turn_restriction_file
        );

        let m: Arc<dyn FrontierModelService> = Arc::new(TurnRestrictionFrontierService {
            restricted_edge_pairs: Arc::new(restricted_edges),
        });
        Ok(m)
    }
}
