use std::sync::Arc;

use routee_compass_core::model::{
    filter::{FilterModel, FilterModelError, FilterModelService},
    state::StateModel,
};
use uom::si::f64::Ratio;

use super::BatteryFilter;

pub struct BatteryFilterService {
    pub soc_lower_bound: Ratio,
}

impl FilterModelService for BatteryFilterService {
    fn build(
        &self,
        _query: &serde_json::Value,
        _state_model: Arc<StateModel>,
    ) -> Result<Arc<dyn FilterModel>, FilterModelError> {
        let model = BatteryFilter {
            soc_lower_bound: self.soc_lower_bound,
        };
        Ok(Arc::new(model))
    }
}
