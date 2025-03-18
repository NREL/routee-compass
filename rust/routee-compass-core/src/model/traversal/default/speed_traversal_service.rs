use super::{
    speed_traversal_engine::SpeedTraversalEngine, speed_traversal_model::SpeedTraversalModel,
};
use crate::model::{
    traversal::{
        traversal_model::TraversalModel, traversal_model_error::TraversalModelError,
        traversal_model_service::TraversalModelService,
    },
    unit::{Speed, SpeedUnit},
};
use std::{str::FromStr, sync::Arc};

pub struct SpeedLookupService {
    pub e: Arc<SpeedTraversalEngine>,
}

impl TraversalModelService for SpeedLookupService {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModel>, TraversalModelError> {
        let max_speed_tuple = match parameters.get("max_speed") {
            Some(max_speed) => {
                let max_speed = max_speed
                    .as_f64()
                    .ok_or_else(|| {
                        TraversalModelError::BuildError("max_speed must be a float".to_string())
                    })
                    .map(Speed::new)?;
                let max_speed_unit = match parameters.get("max_speed_unit") {
                    Some(msu) => {
                        let max_speed_unit_str = msu.as_str().ok_or_else(|| {
                            TraversalModelError::BuildError(
                                "max_speed_unit must be a string".to_string(),
                            )
                        })?;
                        let max_speed_unit =
                            SpeedUnit::from_str(max_speed_unit_str).map_err(|_| {
                                TraversalModelError::BuildError(format!(
                                    "max_speed_unit {} is not a valid speed unit",
                                    max_speed_unit_str
                                ))
                            })?;
                        Ok(max_speed_unit)
                    }
                    None => Err(TraversalModelError::BuildError(
                        "max_speed_unit must be provided if max_speed is provided".to_string(),
                    )),
                }?;
                Some((max_speed, max_speed_unit))
            }
            None => None,
        };

        Ok(Arc::new(SpeedTraversalModel::new(
            self.e.clone(),
            max_speed_tuple,
        )))
    }
}
