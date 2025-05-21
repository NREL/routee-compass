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
        let speed_limit_tuple = match parameters.get("speed_limit") {
            Some(speed_limit) => {
                let speed_limit = speed_limit
                    .as_f64()
                    .ok_or_else(|| {
                        TraversalModelError::BuildError(
                            "key `speed_limit` must be a float".to_string(),
                        )
                    })
                    .map(Speed::from)?;
                let max_speed_unit = match parameters.get("speed_limit_unit") {
                    Some(msu) => {
                        let max_speed_unit_str = msu.as_str().ok_or_else(|| {
                            TraversalModelError::BuildError(
                                "key `speed_limit_unit` must be a string".to_string(),
                            )
                        })?;
                        let max_speed_unit =
                            SpeedUnit::from_str(max_speed_unit_str).map_err(|_| {
                                TraversalModelError::BuildError(format!(
                                    "key `speed_limit_unit` {} is not a valid speed unit",
                                    max_speed_unit_str
                                ))
                            })?;
                        Ok(max_speed_unit)
                    }
                    None => Err(TraversalModelError::BuildError(
                        "key `speed_limit_unit` must be provided if key `speed_limit` is provided"
                            .to_string(),
                    )),
                }?;
                Some((speed_limit, max_speed_unit))
            }
            None => None,
        };

        let model = SpeedTraversalModel::new(self.e.clone(), speed_limit_tuple)?;
        Ok(Arc::new(model))
    }
}
