use std::sync::Arc;

use routee_compass_core::model::constraint::{
    ConstraintModelBuilder, ConstraintModelError, ConstraintModelService,
};
use uom::si::f64::Ratio;

use super::BatteryFilterService;

pub struct BatteryFilterBuilder {
    pub soc_lower_bound: Ratio,
}

impl Default for BatteryFilterBuilder {
    fn default() -> Self {
        BatteryFilterBuilder {
            soc_lower_bound: Ratio::new::<uom::si::ratio::percent>(0.0),
        }
    }
}

impl ConstraintModelBuilder for BatteryFilterBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn ConstraintModelService>, ConstraintModelError> {
        // get the 'soc_lower_bound' from the parameters if it's there, otherwise use the existing value
        let soc_lower_bound = if let Some(soc_lower_bound_percent) = parameters
            .get("soc_lower_bound_percent")
            .and_then(|v| v.as_f64())
        {
            Ratio::new::<uom::si::ratio::percent>(soc_lower_bound_percent)
        } else {
            self.soc_lower_bound
        };
        let service = BatteryFilterService { soc_lower_bound };
        Ok(Arc::new(service))
    }
}
