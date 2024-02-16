use super::weight_heuristic::WeightHeuristic;
use crate::plugin::input::input_json_extensions::InputJsonExtensions;
use crate::plugin::input::input_plugin::InputPlugin;
use crate::plugin::plugin_error::PluginError;

pub struct LoadBalancerPlugin {
    pub heuristic: WeightHeuristic,
}

impl InputPlugin for LoadBalancerPlugin {
    fn process(&self, query: &mut serde_json::Value) -> Result<(), PluginError> {
        let w = self.heuristic.estimate_weight(query)?;
        let mut updated = query.clone();
        query.add_query_weight_estimate(w)?;
        Ok(())
    }
}
