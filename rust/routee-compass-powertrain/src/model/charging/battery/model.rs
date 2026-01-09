use routee_compass_core::model::{
    constraint::{ConstraintModel, ConstraintModelError},
    network::Edge,
    state::{StateModel, StateVariable},
};
use uom::si::f64::Ratio;

#[derive(Clone)]
pub struct BatteryFilter {
    pub soc_lower_bound: Ratio,
    pub state_model_contains_trip_soc: bool,
    pub trip_soc_idx: Option<usize>,
}

impl ConstraintModel for BatteryFilter {
    fn valid_frontier(
        &self,
        _edge: &Edge,
        _previous_edge: Option<&Edge>,
        state: &[StateVariable],
        state_model: &StateModel,
    ) -> Result<bool, ConstraintModelError> {
        if !self.state_model_contains_trip_soc {
            // if we don't have the trip_soc, then this frontier is valid
            return Ok(true);
        }
        if let Some(idx) = self.trip_soc_idx {
            let soc: Ratio = state_model.get_ratio_by_index(state, idx)?;
            return Ok(soc > self.soc_lower_bound);
        }
        Ok(true)
    }

    fn valid_edge(&self, _edge: &Edge) -> Result<bool, ConstraintModelError> {
        Ok(true)
    }
}
