use std::sync::Arc;

use routee_compass_core::model::{
    filter::{FilterModel, FilterModelError, FilterModelService},
    state::StateModel,
};
use uom::si::f64::Ratio;

use super::BatteryFrontier;

pub struct BatteryFrontierService {
    pub soc_lower_bound: Ratio,
}

impl FilterModelService for BatteryFrontierService {
    fn build(
        &self,
        _query: &serde_json::Value,
        _state_model: Arc<StateModel>,
    ) -> Result<Arc<dyn FilterModel>, FilterModelError> {
        let model = BatteryFrontier {
            soc_lower_bound: self.soc_lower_bound,
        };
        Ok(Arc::new(model))
    }
}
