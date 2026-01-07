use super::json_extensions::TraversalJsonField;
use super::traversal_output_format::TraversalOutputFormat;
use crate::app::compass::CompassAppError;
use crate::app::search::SearchAppResult;
use crate::plugin::output::output_plugin::OutputPlugin;
use crate::plugin::output::OutputPluginError;
use routee_compass_core::algorithm::search::EdgeTraversal;
use routee_compass_core::algorithm::search::SearchInstance;
use routee_compass_core::model::cost::TraversalCost;
use routee_compass_core::model::state::StateVariable;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SummaryOp {
    Sum,
    Avg,
    Last,
    First,
    Min,
    Max,
}

pub struct TraversalPlugin {
    route: Option<TraversalOutputFormat>,
    tree: Option<TraversalOutputFormat>,
    summary_ops: HashMap<String, SummaryOp>,
    route_key: String,
    tree_key: String,
}

impl TraversalPlugin {
    pub fn new(
        route: Option<TraversalOutputFormat>,
        tree: Option<TraversalOutputFormat>,
        summary_ops: HashMap<String, SummaryOp>,
    ) -> Result<TraversalPlugin, OutputPluginError> {
        let route_key = TraversalJsonField::RouteOutput.to_string();
        let tree_key = TraversalJsonField::TreeOutput.to_string();
        Ok(TraversalPlugin {
            route,
            tree,
            summary_ops,
            route_key,
            tree_key,
        })
    }
}

impl OutputPlugin for TraversalPlugin {
    fn process(
        &self,
        output: &mut serde_json::Value,
        search_result: &Result<(SearchAppResult, SearchInstance), CompassAppError>,
    ) -> Result<(), OutputPluginError> {
        let (result, si) = match search_result {
            Err(_) => return Ok(()),
            Ok((result, si)) => (result, si),
        };

        // output route if configured
        if let Some(route_args) = self.route {
            let routes_serialized = result
                .routes
                .iter()
                .map(|route| {
                    // construct_route_output(route, si, &route_args, &self.geoms)
                    construct_route_output(route, si, &route_args, &self.summary_ops)
                })
                .collect::<Result<Vec<_>, _>>()
                .map_err(OutputPluginError::OutputPluginFailed)?;

            // vary the type of value stored at the route key. if there is
            // no route, store 'null'. if one, store an output object. if
            // more, store an array of objects.
            let routes_json = match routes_serialized.as_slice() {
                [] => serde_json::Value::Null,
                [route] => route.to_owned(),
                _ => json![routes_serialized],
            };
            output[&self.route_key] = routes_json;
        }

        // output tree(s) if configured
        if let Some(tree_args) = self.tree {
            let trees_serialized = result
                .trees
                .iter()
                .map(|tree| {
                    // tree_args.generate_tree_output(tree, &self.geoms)
                    tree_args.generate_tree_output(
                        tree,
                        si.map_model.clone(),
                        si.state_model.clone(),
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            let trees_json = match trees_serialized.as_slice() {
                [] => serde_json::Value::Null,
                [tree] => tree.to_owned(),
                _ => json![trees_serialized],
            };
            output[&self.tree_key] = json![trees_json];
        }

        Ok(())
    }
}

/// creates the JSON output for a route.
fn construct_route_output(
    route: &Vec<EdgeTraversal>,
    si: &SearchInstance,
    output_format: &TraversalOutputFormat,
    summary_ops: &HashMap<String, SummaryOp>,
) -> Result<serde_json::Value, String> {
    let last_edge = route
        .last()
        .ok_or_else(|| String::from("cannot find result route state when route is empty"))?;
    let path_json = output_format
        .generate_route_output(route, si.map_model.clone(), si.state_model.clone())
        .map_err(|e| e.to_string())?;
    let final_state = si
        .state_model
        .serialize_state(&last_edge.result_state, true)
        .map_err(|e| format!("failed serializing final trip state: {e}"))?;

    log::debug!("state model: {:?}", si.state_model);
    log::debug!("final state: {final_state:?}");
    log::debug!("result state: {:?}", last_edge.result_state);

    let state_model = si.state_model.serialize_state_model();

    // Compute total route cost by summing all edge costs
    let route_cost = route
        .iter()
        .fold(TraversalCost::default(), |mut acc, edge| {
            acc.total_cost += edge.cost.total_cost;
            acc.objective_cost += edge.cost.objective_cost;
            acc
        });

    let cost = json![route_cost];
    let cost_model = si
        .cost_model
        .serialize_cost_info()
        .map_err(|e| e.to_string())?;

    let mut traversal_summary = serde_json::Map::new();
    for (i, (name, feature)) in si.state_model.indexed_iter() {
        let op = summary_ops.get(name).cloned().unwrap_or_else(|| {
            if feature.is_accumulator() {
                SummaryOp::Last
            } else {
                SummaryOp::Sum
            }
        });

        let value = match op {
            SummaryOp::Sum => route.iter().map(|e| e.result_state[i]).sum(),
            SummaryOp::Avg => {
                let sum: StateVariable = route.iter().map(|e| e.result_state[i]).sum();
                let count = route.len() as f64;
                StateVariable(sum.0 / count)
            }
            SummaryOp::Last => last_edge.result_state[i],
            SummaryOp::First => route
                .first()
                .map(|e| e.result_state[i])
                .unwrap_or(StateVariable::ZERO),
            SummaryOp::Min => route
                .iter()
                .map(|e| e.result_state[i])
                .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or(StateVariable::ZERO),
            SummaryOp::Max => route
                .iter()
                .map(|e| e.result_state[i])
                .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or(StateVariable::ZERO),
        };
        let serialized = feature
            .serialize_variable(&value)
            .map_err(|e| e.to_string())?;
        traversal_summary.insert(name.clone(), serialized);
    }

    let result = serde_json::json![{
        "final_state": final_state,
        "state_model": state_model,
        "cost_model": cost_model,
        "cost": cost,
        "path": path_json,
        "traversal_summary": traversal_summary
    }];
    Ok(result)
}
