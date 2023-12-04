use crate::{
    app::compass::config::{
        builders::InputPluginBuilder, compass_configuration_error::CompassConfigurationError,
        config_json_extension::ConfigJsonExtensions,
    },
    plugin::input::input_plugin::InputPlugin,
};

use super::{plugin::LoadBalancerPlugin, weight_heuristic::WeightHeuristic};

pub struct LoadBalancerBuilder {}

impl InputPluginBuilder for LoadBalancerBuilder {
    fn build(
        &self,
        params: &serde_json::Value,
    ) -> Result<Box<dyn InputPlugin>, CompassConfigurationError> {
        let heuristic = params.get_config_serde::<WeightHeuristic>(
            String::from("weight_heuristic"),
            String::from("load_balancer"),
        )?;
        Ok(Box::new(LoadBalancerPlugin { heuristic }))
    }
}
