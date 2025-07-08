use super::json_extensions::TraversalJsonField;
use super::traversal_output_format::TraversalOutputFormat;
use crate::app::compass::CompassAppError;
use crate::app::search::SearchAppResult;
use crate::plugin::output::output_plugin::OutputPlugin;
use crate::plugin::output::OutputPluginError;
use routee_compass_core::algorithm::search::EdgeTraversal;
use routee_compass_core::algorithm::search::SearchInstance;
use serde_json::json;

pub struct TraversalPlugin {
    route: Option<TraversalOutputFormat>,
    tree: Option<TraversalOutputFormat>,
    route_key: String,
    tree_key: String,
}

impl TraversalPlugin {
    pub fn new(
        route: Option<TraversalOutputFormat>,
        tree: Option<TraversalOutputFormat>,
    ) -> Result<TraversalPlugin, OutputPluginError> {
        let route_key = TraversalJsonField::RouteOutput.to_string();
        let tree_key = TraversalJsonField::TreeOutput.to_string();
        Ok(TraversalPlugin {
            route,
            tree,
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
                    construct_route_output(route, si, &route_args)
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
                        si.cost_model.clone(),
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
) -> Result<serde_json::Value, String> {
    let last_edge = route
        .last()
        .ok_or_else(|| String::from("cannot find result route state when route is empty"))?;
    let path_json = output_format
        .generate_route_output(
            route,
            si.map_model.clone(),
            si.state_model.clone(),
            si.cost_model.clone(),
        )
        .map_err(|e| e.to_string())?;
    let traversal_summary = si.state_model.serialize_state(&last_edge.result_state);

    log::debug!("state model: {:?}", si.state_model);
    log::debug!("traversal summary: {:?}", traversal_summary);
    log::debug!("result state: {:?}", last_edge.result_state);

    let state_model = si.state_model.serialize_state_model();
    let cost = si
        .cost_model
        .serialize_cost(&last_edge.result_state, si.state_model.clone())
        .map_err(|e| e.to_string())?;
    let cost_model = si
        .cost_model
        .serialize_cost_info()
        .map_err(|e| e.to_string())?;
    let result = serde_json::json![{
        "traversal_summary": traversal_summary,
        "state_model": state_model,
        "cost_model": cost_model,
        "cost": cost,
        "path": path_json
    }];
    Ok(result)
}
