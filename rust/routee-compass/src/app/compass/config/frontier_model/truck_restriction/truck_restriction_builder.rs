use crate::app::compass::config::{
    compass_configuration_field::CompassConfigurationField,
    config_json_extension::ConfigJsonExtensions,
};
use routee_compass_core::{
    model::{
        frontier::{
            frontier_model_builder::FrontierModelBuilder, frontier_model_error::FrontierModelError,
            frontier_model_service::FrontierModelService,
        },
        road_network::edge_id::EdgeId,
    },
    util::fs::read_utils,
};
use serde::Deserialize;
use std::{collections::HashMap, sync::Arc};

use super::{
    truck_restriction::TruckRestriction, truck_restriction_service::TruckRestrictionFrontierService,
};

#[derive(Debug, Clone, Deserialize)]
struct RestrictionRow {
    edge_id: EdgeId,
    restriction_name: String,
    restriction_value: f64,
    restriction_unit: String,
}

impl RestrictionRow {
    fn to_restriction(&self) -> Result<TruckRestriction, FrontierModelError> {
        // use serde to deserialize the restriction value
        let json = serde_json::json!({
            self.restriction_name.clone(): (self.restriction_value, self.restriction_unit.clone())
        });
        let restriction: TruckRestriction = serde_json::from_value(json).map_err(|e| {
            FrontierModelError::BuildError(format!(
                "Unable to deserialize restriction {:?} due to: {}",
                self, e
            ))
        })?;
        Ok(restriction)
    }
}

pub struct TruckRestrictionBuilder {}

impl FrontierModelBuilder for TruckRestrictionBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn FrontierModelService>, FrontierModelError> {
        let frontier_key = CompassConfigurationField::Frontier.to_string();
        let truck_restriction_input_file_key = String::from("truck_restriction_input_file");

        let truck_restriction_input_file = parameters
            .get_config_path(&truck_restriction_input_file_key, &frontier_key)
            .map_err(|e| {
                FrontierModelError::BuildError(format!(
                    "configuration error due to {}: {}",
                    truck_restriction_input_file_key.clone(),
                    e
                ))
            })?;

        let rows: Vec<RestrictionRow> =
            read_utils::from_csv(&truck_restriction_input_file, true, None)
                .map_err(|e| {
                    FrontierModelError::BuildError(format!(
                        "configuration error due to {}: {}",
                        truck_restriction_input_file_key.clone(),
                        e
                    ))
                })?
                .to_vec();

        let mut truck_restriction_lookup: HashMap<EdgeId, Vec<TruckRestriction>> = HashMap::new();
        for row in rows {
            let restriction: TruckRestriction = row.clone().to_restriction()?;
            let restrictions = truck_restriction_lookup
                .entry(row.edge_id)
                .or_default();
            restrictions.push(restriction);
        }

        let m = TruckRestrictionFrontierService {
            truck_restriction_lookup: Arc::new(truck_restriction_lookup),
        };

        Ok(Arc::new(m))
    }
}
