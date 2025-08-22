use std::sync::Arc;

use uom::si::f64::ThermodynamicTemperature;

use crate::model::traversal::{
    default::temperature::TemperatureTraversalModel, TraversalModel, TraversalModelError,
    TraversalModelService,
};

pub struct TemperatureTraversalService {}

impl TraversalModelService for TemperatureTraversalService {
    fn build(
        &self,
        query: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModel>, TraversalModelError> {
        let ambient_temperature_f = query
            .get("ambient_temperature_f")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| {
                TraversalModelError::TraversalModelFailure(
                    "Missing or invalid ambient_temperature parameter".to_string(),
                )
            })?;

        let ambient_temperature = ThermodynamicTemperature::new::<
            uom::si::thermodynamic_temperature::degree_fahrenheit,
        >(ambient_temperature_f);

        Ok(Arc::new(TemperatureTraversalModel {
            ambient_temperature,
        }))
    }
}
