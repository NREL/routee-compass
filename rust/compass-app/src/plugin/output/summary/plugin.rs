use crate::plugin::plugin_error::PluginError;

use compass_core::{
    algorithm::search::{edge_traversal::EdgeTraversal, search_error::SearchError},
    model::cost::cost::Cost,
};
use serde_json;
use uom::si;

use crate::plugin::output::OutputPlugin;

use super::json_extensions::SummaryJsonExtensions;


pub fn build_summary_output_plugin() -> Result<OutputPlugin, PluginError> {
    let summary_plugin = move |output: &serde_json::Value,
                               search_result: Result<&Vec<EdgeTraversal>, SearchError>|
          -> Result<serde_json::Value, PluginError> {
        let mut updated_output = output.clone();
        let route = search_result?;
        let cost = route 
            .iter()
            .map(|traversal| traversal.edge_cost())
            .sum::<Cost>();
        updated_output.add_cost(cost)?;
        let distance = route 
            .iter()
            .map(|traversal| traversal.edge.distance.get::<si::length::kilometer>())
            .sum::<f64>();
        updated_output.add_distance(distance)?;
        Ok(updated_output)
    };
    Ok(Box::new(summary_plugin))
}

