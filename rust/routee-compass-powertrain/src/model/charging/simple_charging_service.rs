use std::{collections::HashSet, str::FromStr, sync::Arc};

use routee_compass_core::model::traversal::{
    TraversalModel, TraversalModelError, TraversalModelService,
};
use uom::si::f64::Ratio;

use crate::model::{
    charging::{
        charging_station_locator::{ChargingStationLocator, PowerType},
        simple_charging_model::SimpleChargingModel,
    },
    energy_model_ops::get_query_start_soc,
};

pub struct SimpleChargingService {
    pub charging_station_locator: Arc<ChargingStationLocator>,
    pub starting_soc: Ratio,
    pub full_soc: Ratio,
    pub charge_soc_threshold: Ratio,
    pub valid_power_types: HashSet<PowerType>,
}

impl TraversalModelService for SimpleChargingService {
    fn build(
        &self,
        query: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModel>, TraversalModelError> {
        let starting_soc = match get_query_start_soc(query)? {
            Some(soc) => soc,
            None => self.starting_soc,
        };
        // get 'full_soc_percent' from the query if it's there otherwise use the existing value
        let full_soc = if let Some(full_soc_percent) =
            query.get("full_soc_percent").and_then(|v| v.as_f64())
        {
            Ratio::new::<uom::si::ratio::percent>(full_soc_percent)
        } else {
            self.full_soc
        };

        let charge_soc_threshold = if let Some(charge_soc_threshold_percent) = query
            .get("charge_soc_threshold_percent")
            .and_then(|v| v.as_f64())
        {
            Ratio::new::<uom::si::ratio::percent>(charge_soc_threshold_percent)
        } else {
            self.charge_soc_threshold
        };

        // get the valid power types from the query if they are provided, otherwise use existing values
        let valid_power_types = if let Some(valid_power_types_str) = query
            .get("valid_power_types")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<&str>>())
        {
            valid_power_types_str
                .into_iter()
                .map(|s| {
                    PowerType::from_str(s).map_err(|_| {
                        TraversalModelError::BuildError(format!(
                            "Invalid power type: '{s}'. Valid power types are: l1, l2, dcfc"
                        ))
                    })
                })
                .collect::<Result<HashSet<PowerType>, TraversalModelError>>()?
        } else {
            self.valid_power_types.clone()
        };

        let model = SimpleChargingModel {
            charging_station_locator: self.charging_station_locator.clone(),
            starting_soc,
            full_soc,
            charge_soc_threshold,
            valid_power_types,
        };
        Ok(Arc::new(model))
    }
}
