use super::{plugin::LoadBalancerPlugin, weight_heuristic::WeightHeuristic};
use routee_compass_core::config::{CompassConfigurationError, ConfigJsonExtensions};
use crate::plugin::input::{
    InputPlugin, InputPluginBuilder,
};
use std::sync::Arc;

pub struct LoadBalancerBuilder {}

impl InputPluginBuilder for LoadBalancerBuilder {
    fn build(
        &self,
        params: &serde_json::Value,
    ) -> Result<Arc<dyn InputPlugin>, CompassConfigurationError> {
        let heuristic =
            params.get_config_serde::<WeightHeuristic>(&"weight_heuristic", &"load_balancer")?;
        Ok(Arc::new(LoadBalancerPlugin { heuristic }))
    }
}
