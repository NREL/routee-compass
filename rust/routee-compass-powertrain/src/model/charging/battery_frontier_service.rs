use std::sync::Arc;

use routee_compass_core::model::{
    frontier::{FrontierModel, FrontierModelError, FrontierModelService},
    state::StateModel,
};
use uom::si::f64::Ratio;

use crate::model::charging::battery_frontier_model::BatteryFrontier;

pub struct BatteryFrontierService {
    pub soc_lower_bound: Ratio,
}

impl FrontierModelService for BatteryFrontierService {
    fn build(
        &self,
        _query: &serde_json::Value,
        _state_model: Arc<StateModel>,
    ) -> Result<Arc<dyn FrontierModel>, FrontierModelError> {
        let model = BatteryFrontier {
            soc_lower_bound: self.soc_lower_bound,
        };
        Ok(Arc::new(model))
    }
}
