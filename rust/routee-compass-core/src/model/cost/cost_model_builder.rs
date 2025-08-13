use super::cost_model_service::CostModelService;
use crate::config::{CompassConfigurationError};
use crate::model::cost::{CostModelConfig};
use std::{sync::Arc};

pub struct CostModelBuilder {}

impl CostModelBuilder {
    pub fn build(
        &self,
        config: &serde_json::Value,
    ) -> Result<CostModelService, CompassConfigurationError> {
        let conf: CostModelConfig = serde_json::from_value(config.clone())?;
        let model = CostModelService {
            vehicle_rates: Arc::new(conf.get_vehicle_rates()),
            network_rates: Arc::new(conf.get_network_rates()),
            weights: Arc::new(conf.get_weights()),
            cost_aggregation: conf.get_cost_aggregation(),
            ignore_unknown_weights: conf.get_ignore_policy(),
        };
        Ok(model)
    }
}
