use routee_compass_core::model::{
    filter::{FilterModel, FilterModelError},
    network::Edge,
    state::{StateModel, StateVariable},
};
use uom::si::f64::Ratio;

use crate::model::fieldname;

#[derive(Clone)]
pub struct BatteryFrontier {
    pub soc_lower_bound: Ratio,
}

impl FilterModel for BatteryFrontier {
    fn valid_frontier(
        &self,
        _edge: &Edge,
        _previous_edge: Option<&Edge>,
        state: &[StateVariable],
        state_model: &StateModel,
    ) -> Result<bool, FilterModelError> {
        if !state_model.contains_key(&fieldname::TRIP_SOC.to_string()) {
            // if we don't have the trip_soc, then this frontier is valid
            return Ok(true);
        }
        let soc: Ratio = state_model.get_ratio(state, fieldname::TRIP_SOC).map_err(|_| {
            FilterModelError::FilterModelError(
                "BatteryFrontier filter model requires the state variable 'trip_soc' but not found".to_string(),
            )
        })?;
        Ok(soc > self.soc_lower_bound)
    }

    fn valid_edge(&self, _edge: &Edge) -> Result<bool, FilterModelError> {
        Ok(true)
    }
}
