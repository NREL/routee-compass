use std::sync::Arc;

use routee_compass_core::model::{
    constraint::{ConstraintModel, ConstraintModelError, ConstraintModelService},
    state::StateModel,
};
use uom::si::f64::Ratio;

use crate::model::fieldname;

use super::BatteryFilter;

pub struct BatteryFilterService {
    pub soc_lower_bound: Ratio,
}

impl ConstraintModelService for BatteryFilterService {
    fn build(
        &self,
        _query: &serde_json::Value,
        state_model: Arc<StateModel>,
    ) -> Result<Arc<dyn ConstraintModel>, ConstraintModelError> {
        let state_model_contains_trip_soc =
            state_model.contains_key(&fieldname::TRIP_SOC.to_string());
        let model = BatteryFilter {
            soc_lower_bound: self.soc_lower_bound,
            state_model_contains_trip_soc,
        };
        Ok(Arc::new(model))
    }
}
