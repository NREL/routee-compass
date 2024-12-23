use std::sync::Arc;

use super::weight_heuristic::WeightHeuristic;
use crate::app::search::search_app::SearchApp;
use crate::plugin::input::input_json_extensions::InputJsonExtensions;
use crate::plugin::input::input_plugin::InputPlugin;
use crate::plugin::input::InputPluginError;

pub struct LoadBalancerPlugin {
    pub heuristic: WeightHeuristic,
}

impl InputPlugin for LoadBalancerPlugin {
    fn process(
        &self,
        query: &mut serde_json::Value,
        _search_app: Arc<SearchApp>,
    ) -> Result<(), InputPluginError> {
        let w = self.heuristic.estimate_weight(query)?;
        let _updated = query.clone();
        query.add_query_weight_estimate(w)?;
        Ok(())
    }
}
