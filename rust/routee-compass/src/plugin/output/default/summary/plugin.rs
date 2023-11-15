use super::json_extensions::SummaryJsonExtensions;
use crate::app::{
    compass::compass_app_error::CompassAppError, search::search_app_result::SearchAppResult,
};
use crate::plugin::output::output_plugin::OutputPlugin;
use crate::plugin::plugin_error::PluginError;
use routee_compass_core::model::cost::Cost;
use serde_json;

pub struct SummaryOutputPlugin {}

impl OutputPlugin for SummaryOutputPlugin {
    /// append "Cost" value to the output JSON
    fn process(
        &self,
        output: &serde_json::Value,
        search_result: &Result<SearchAppResult, CompassAppError>,
    ) -> Result<Vec<serde_json::Value>, PluginError> {
        match search_result {
            Err(_e) => Ok(vec![output.clone()]),
            Ok(result) => {
                let mut updated_output = output.clone();
                let cost = result
                    .route
                    .iter()
                    .map(|traversal| traversal.edge_cost())
                    .sum::<Cost>();
                updated_output.add_cost(cost)?;
                Ok(vec![updated_output])
            }
        }
    }
}

#[cfg(test)]

mod tests {
    use std::{collections::HashMap, time::Duration};

    use chrono::Local;
    use routee_compass_core::{
        algorithm::search::edge_traversal::EdgeTraversal,
        model::{road_network::edge_id::EdgeId, traversal::state::state_variable::StateVar},
    };

    use super::*;

    #[test]
    fn test_summary_output_plugin() {
        let output_result = serde_json::json!({});
        let route = vec![
            EdgeTraversal {
                edge_id: EdgeId(0),
                access_cost: Cost::from(1.0),
                traversal_cost: Cost::from(1.0),
                result_state: vec![StateVar(0.0)],
            },
            EdgeTraversal {
                edge_id: EdgeId(1),
                access_cost: Cost::from(1.0),
                traversal_cost: Cost::from(1.0),
                result_state: vec![StateVar(0.0)],
            },
            EdgeTraversal {
                edge_id: EdgeId(2),
                access_cost: Cost::from(1.0),
                traversal_cost: Cost::from(1.0),
                result_state: vec![StateVar(0.0)],
            },
        ];
        let search_result = SearchAppResult {
            route,
            tree: HashMap::new(),
            search_start_time: Local::now(),
            search_runtime: Duration::ZERO,
            route_runtime: Duration::ZERO,
            total_runtime: Duration::ZERO,
        };
        let summary_plugin = SummaryOutputPlugin {};
        let updated_output = summary_plugin
            .process(&output_result, &Ok(search_result))
            .unwrap();
        let cost: f64 = updated_output[0].get_cost().unwrap().into();
        assert_eq!(cost, 6.0);
    }
}
