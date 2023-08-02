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


#[cfg(test)]

mod tests {
    use compass_core::model::{property::edge::Edge, traversal::state::state_variable::StateVar};

    use super::*;

    #[test]
    fn test_summary_output_plugin(){
        let output_result = serde_json::json!({});
        let route = vec![
            EdgeTraversal {
                edge: Edge::default(),
                access_cost: Cost::from(1.0),
                traversal_cost: Cost::from(1.0),
                result_state: vec![vec![StateVar(0.0)]],
            },
            EdgeTraversal {
                edge: Edge::default(),
                access_cost: Cost::from(1.0),
                traversal_cost: Cost::from(1.0),
                result_state: vec![vec![StateVar(0.0)]],
            },
            EdgeTraversal {
                edge: Edge::default(),
                access_cost: Cost::from(1.0),
                traversal_cost: Cost::from(1.0),
                result_state: vec![vec![StateVar(0.0)]],
            },
        ];
        let summary_plugin = build_summary_output_plugin().unwrap();
        let updated_output = summary_plugin(&output_result, Ok(&route)).unwrap();
        let cost: f64 = updated_output.get_cost().unwrap().into();
        let distance_km: f64 = updated_output.get_distance().unwrap();
        assert_eq!(cost, 6.0);

        // each default edge has a lenght of 1 km
        assert_eq!(distance_km, 3.0);
    }
}
