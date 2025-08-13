use super::{
    RestrictionRow, VehicleParameterType, VehicleRestriction, VehicleRestrictionFrontierService,
};
use crate::config::{CompassConfigurationField, ConfigJsonExtensions};
use crate::util::compact_ordered_hash_map::CompactOrderedHashMap;
use crate::{
    model::{
        frontier::{FrontierModelBuilder, FrontierModelError, FrontierModelService},
        network::edge_id::EdgeId,
    },
    util::fs::read_utils,
};
use kdam::Bar;
use std::{collections::HashMap, path::PathBuf, sync::Arc};

pub struct VehicleRestrictionBuilder {}

impl FrontierModelBuilder for VehicleRestrictionBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn FrontierModelService>, FrontierModelError> {
        let frontier_key = CompassConfigurationField::Frontier.to_string();
        let vehicle_restriction_input_file_key = String::from("vehicle_restriction_input_file");

        let vehicle_restriction_input_file = parameters
            .get_config_path(&vehicle_restriction_input_file_key, &frontier_key)
            .map_err(|e| {
                FrontierModelError::BuildError(format!(
                    "configuration error due to {}: {}",
                    vehicle_restriction_input_file_key.clone(),
                    e
                ))
            })?;

        let vehicle_restriction_lookup =
            vehicle_restriction_lookup_from_file(&vehicle_restriction_input_file)?;

        let m = VehicleRestrictionFrontierService {
            vehicle_restriction_lookup: Arc::new(vehicle_restriction_lookup),
        };

        Ok(Arc::new(m))
    }
}

pub fn vehicle_restriction_lookup_from_file(
    vehicle_restriction_input_file: &PathBuf,
) -> Result<
    HashMap<EdgeId, CompactOrderedHashMap<VehicleParameterType, VehicleRestriction>>,
    FrontierModelError,
> {
    let rows: Vec<RestrictionRow> = read_utils::from_csv(
        &vehicle_restriction_input_file,
        true,
        Some(Bar::builder().desc("vehicle restrictions")),
        None,
    )
    .map_err(|e| {
        FrontierModelError::BuildError(format!(
            "Could not load vehicle restriction file {vehicle_restriction_input_file:?}: {e}"
        ))
    })?
    .to_vec();

    let mut vehicle_restriction_lookup: HashMap<
        EdgeId,
        CompactOrderedHashMap<VehicleParameterType, VehicleRestriction>,
    > = HashMap::new();
    for row in rows {
        let restriction = VehicleRestriction::try_from(&row)?;
        match vehicle_restriction_lookup.get_mut(&row.edge_id) {
            None => {
                let mut restrictions = CompactOrderedHashMap::empty();
                restrictions.insert(restriction.vehicle_parameter_type().clone(), restriction);
                vehicle_restriction_lookup.insert(row.edge_id, restrictions);
            }
            Some(restrictions) => {
                restrictions.insert(restriction.vehicle_parameter_type().clone(), restriction);
            }
        }
    }
    Ok(vehicle_restriction_lookup)
}
