use super::turn_restriction_service::{RestrictedEdgePair, TurnRestrictionFrontierService};
use crate::config::{CompassConfigurationField, ConfigJsonExtensions};
use crate::{
    model::constraint::{ConstraintModelBuilder, ConstraintModelError, ConstraintModelService},
    util::fs::read_utils,
};
use kdam::Bar;
use std::{collections::HashSet, sync::Arc};

pub struct TurnRestrictionBuilder {}

impl ConstraintModelBuilder for TurnRestrictionBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn ConstraintModelService>, ConstraintModelError> {
        let constraint_key = CompassConfigurationField::Constraint.to_string();
        let turn_restriction_file_key = String::from("turn_restriction_input_file");

        let turn_restriction_file = parameters
            .get_config_path(&turn_restriction_file_key, &constraint_key)
            .map_err(|e| {
                ConstraintModelError::BuildError(format!(
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
            ConstraintModelError::BuildError(format!(
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

        let m: Arc<dyn ConstraintModelService> = Arc::new(TurnRestrictionFrontierService {
            restricted_edge_pairs: Arc::new(restricted_edges),
        });
        Ok(m)
    }
}
