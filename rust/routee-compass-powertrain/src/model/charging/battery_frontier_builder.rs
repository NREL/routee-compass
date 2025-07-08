use std::sync::Arc;

use routee_compass_core::model::frontier::{
    FrontierModelBuilder, FrontierModelError, FrontierModelService,
};
use uom::si::f64::Ratio;

use crate::model::charging::battery_frontier_service::BatteryFrontierService;

pub struct BatteryFrontierBuilder {
    pub soc_lower_bound: Ratio,
}

impl Default for BatteryFrontierBuilder {
    fn default() -> Self {
        BatteryFrontierBuilder {
            soc_lower_bound: Ratio::new::<uom::si::ratio::percent>(0.0),
        }
    }
}

impl FrontierModelBuilder for BatteryFrontierBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn FrontierModelService>, FrontierModelError> {
        // get the 'soc_lower_bound' from the parameters if it's there, otherwise use the existing value
        let soc_lower_bound = if let Some(soc_lower_bound_percent) = parameters
            .get("soc_lower_bound_percent")
            .and_then(|v| v.as_f64())
        {
            Ratio::new::<uom::si::ratio::percent>(soc_lower_bound_percent)
        } else {
            self.soc_lower_bound
        };
        let service = BatteryFrontierService { soc_lower_bound };
        Ok(Arc::new(service))
    }
}
