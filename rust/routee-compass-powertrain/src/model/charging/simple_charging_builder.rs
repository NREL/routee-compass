use std::{collections::HashSet, str::FromStr, sync::Arc};

use routee_compass_core::{
    config::ConfigJsonExtensions,
    model::{
        map::DistanceTolerance,
        traversal::{TraversalModelBuilder, TraversalModelError, TraversalModelService},
    },
};
use uom::si::f64::Ratio;

use crate::model::charging::{
    charging_station_locator::{ChargingStationLocator, PowerType},
    simple_charging_service::SimpleChargingService,
};

pub struct SimpleChargingBuilder {
    full_soc: Ratio,
    starting_soc: Ratio,
    charge_soc_threshold: Ratio,
    valid_power_types: HashSet<PowerType>,
}

impl Default for SimpleChargingBuilder {
    fn default() -> Self {
        SimpleChargingBuilder {
            full_soc: Ratio::new::<uom::si::ratio::percent>(100.0),
            starting_soc: Ratio::new::<uom::si::ratio::percent>(100.0),
            charge_soc_threshold: Ratio::new::<uom::si::ratio::percent>(20.0),
            valid_power_types: HashSet::from([PowerType::L1, PowerType::L2, PowerType::DCFC]),
        }
    }
}

impl TraversalModelBuilder for SimpleChargingBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModelService>, TraversalModelError> {
        let full_soc = if let Some(full_soc_percent) =
            parameters.get("full_soc_percent").and_then(|v| v.as_f64())
        {
            Ratio::new::<uom::si::ratio::percent>(full_soc_percent)
        } else {
            self.full_soc
        };

        let charge_soc_threshold = if let Some(charge_soc_threshold_percent) = parameters
            .get("charge_soc_threshold_percent")
            .and_then(|v| v.as_f64())
        {
            Ratio::new::<uom::si::ratio::percent>(charge_soc_threshold_percent)
        } else {
            self.charge_soc_threshold
        };

        let valid_power_types = if let Some(valid_power_types_str) = parameters
            .get("valid_power_types")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<&str>>())
        {
            valid_power_types_str
                .into_iter()
                .map(|s| {
                    PowerType::from_str(s).map_err(|_| {
                        TraversalModelError::BuildError(format!(
                            "Invalid power type: '{}'. Valid power types are: l1, l2, dcfc",
                            s
                        ))
                    })
                })
                .collect::<Result<HashSet<PowerType>, TraversalModelError>>()?
        } else {
            self.valid_power_types.clone()
        };

        let charging_station_input_file = parameters.get_config_path(&"charging_station_input_file", &"simple charging model")
            .map_err(|e| {
                TraversalModelError::BuildError(format!(
                    "failure reading 'charging_station_input_file' from simple charging model configuration: {}",
                    e
                ))
            })?;

        let vertex_input_file = parameters.get_config_path(&"vertex_input_file", &"simple charging model")
            .map_err(|e| {
                TraversalModelError::BuildError(format!(
                    "failure reading 'vertex_input_file' from simple charging model configuration: {}",
                    e
                ))
            })?;

        let station_match_tolerance: Option<DistanceTolerance> = parameters.get_config_serde_optional(&"station_match_tolerance", &"simple charging model").map_err(|e| {
            TraversalModelError::BuildError(format!(
                "failure reading 'station_match_tolerance' from simple charging model configuration: {}",
                e
            ))
        })?;
        let charging_station_locator = Arc::new(
            ChargingStationLocator::from_csv_files(
                &charging_station_input_file,
                &vertex_input_file,
                station_match_tolerance,
            )
            .map_err(|e| {
                TraversalModelError::BuildError(format!(
                    "failed to load charging station locator: {}",
                    e
                ))
            })?,
        );

        Ok(Arc::new(SimpleChargingService {
            charging_station_locator,
            starting_soc: self.starting_soc,
            full_soc,
            charge_soc_threshold,
            valid_power_types,
        }))
    }
}
