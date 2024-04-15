use super::json_extensions::TraversalJsonField;
use super::traversal_output_format::TraversalOutputFormat;
use crate::app::compass::compass_app_error::CompassAppError;
use crate::app::search::search_app_result::SearchAppResult;
use crate::plugin::output::output_plugin::OutputPlugin;
use crate::plugin::plugin_error::PluginError;
use geo::LineString;
use kdam::Bar;
use kdam::BarExt;
use routee_compass_core::algorithm::search::edge_traversal::EdgeTraversal;
use routee_compass_core::algorithm::search::search_instance::SearchInstance;
use routee_compass_core::util::fs::fs_utils;
use routee_compass_core::util::fs::read_utils::read_raw_file;
use routee_compass_core::util::geo::geo_io_utils;
use serde_json::json;
use std::path::Path;

pub struct TraversalPlugin {
    geoms: Box<[LineString<f32>]>,
    route: Option<TraversalOutputFormat>,
    tree: Option<TraversalOutputFormat>,
    route_key: String,
    tree_key: String,
}

impl TraversalPlugin {
    pub fn from_file<P: AsRef<Path>>(
        filename: &P,
        route: Option<TraversalOutputFormat>,
        tree: Option<TraversalOutputFormat>,
    ) -> Result<TraversalPlugin, PluginError> {
        let count = fs_utils::line_count(filename, fs_utils::is_gzip(filename)).map_err(|e| {
            PluginError::FileReadError(filename.as_ref().to_path_buf(), e.to_string())
        })?;

        let mut pb = Bar::builder()
            .total(count)
            .animation("fillup")
            .desc("geometry file")
            .build()
            .map_err(PluginError::InternalError)?;

        let cb = Box::new(|| {
            let _ = pb.update(1);
        });
        let geoms =
            read_raw_file(filename, geo_io_utils::parse_linestring, Some(cb)).map_err(|e| {
                PluginError::FileReadError(filename.as_ref().to_path_buf(), e.to_string())
            })?;
        println!();

        let route_key = TraversalJsonField::RouteOutput.to_string();
        let tree_key = TraversalJsonField::TreeOutput.to_string();
        Ok(TraversalPlugin {
            geoms,
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
    ) -> Result<(), PluginError> {
        match search_result {
            Err(_) => Ok(()),
            Ok((result, si)) => {
                match self.route {
                    None => {}
                    Some(route_args) => {
                        let routes_serialized = result
                            .routes
                            .iter()
                            .map(|route| {
                                construct_route_output(route, si, &route_args, &self.geoms)
                            })
                            .collect::<Result<Vec<_>, _>>()
                            .map_err(PluginError::PluginFailed)?;

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
                }

                match self.tree {
                    None => {}
                    Some(tree_args) => {
                        let trees_serialized = result
                            .trees
                            .iter()
                            .map(|tree| tree_args.generate_tree_output(tree, &self.geoms))
                            .collect::<Result<Vec<_>, _>>()?;
                        let trees_json = match trees_serialized.as_slice() {
                            [] => serde_json::Value::Null,
                            [tree] => tree.to_owned(),
                            _ => json![trees_serialized],
                        };
                        output[&self.tree_key] = json![trees_json];
                    }
                }

                Ok(())
            }
        }
    }
}

/// creates the JSON output for a route.
fn construct_route_output(
    route: &Vec<EdgeTraversal>,
    si: &SearchInstance,
    output_format: &TraversalOutputFormat,
    geoms: &[LineString<f32>],
) -> Result<serde_json::Value, String> {
    let last_edge = route
        .last()
        .ok_or_else(|| String::from("cannot find result route state when route is empty"))?;
    let path_json = output_format
        .generate_route_output(route, geoms)
        .map_err(|e| e.to_string())?;
    let traversal_summary = si.state_model.serialize_state(&last_edge.result_state);
    let state_model = si.state_model.serialize_state_model();
    let cost = si
        .cost_model
        .serialize_cost(&last_edge.result_state)
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

#[cfg(test)]
mod tests {

    use routee_compass_core::util::{
        fs::read_utils::read_raw_file, geo::geo_io_utils::parse_linestring,
    };

    use std::path::PathBuf;

    fn mock_geometry_file() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("plugin")
            .join("output")
            .join("default")
            .join("test")
            .join("geometry.txt")
    }

    #[test]
    fn test_geometry_deserialization() {
        let result = read_raw_file(mock_geometry_file(), parse_linestring, None).unwrap();
        assert_eq!(result.len(), 3);
    }

    // TODO:
    //   the API for OutputPlugin now expects a SearchInstance which is non-trivial to instantiate.
    //   the logic for adding geometries should be refactored into a separate function and this test
    //   should be moved to the file where that function exists.

    // #[test]
    // fn test_add_geometry() {
    //     let expected_geometry = String::from("LINESTRING(0 0,1 1,2 2,3 3,4 4,5 5,6 6,7 7,8 8)");
    //     let mut output_result = serde_json::json!({});
    //     let route = vec![
    //         EdgeTraversal {
    //             edge_id: EdgeId(0),
    //             access_cost: Cost::from(0.0),
    //             traversal_cost: Cost::from(0.0),
    //             result_state: vec![StateVar(0.0)],
    //         },
    //         EdgeTraversal {
    //             edge_id: EdgeId(1),
    //             access_cost: Cost::from(0.0),
    //             traversal_cost: Cost::from(0.0),
    //             result_state: vec![StateVar(0.0)],
    //         },
    //         EdgeTraversal {
    //             edge_id: EdgeId(2),
    //             access_cost: Cost::from(0.0),
    //             traversal_cost: Cost::from(0.0),
    //             result_state: vec![StateVar(0.0)],
    //         },
    //     ];
    //     let search_result = SearchAppResult {
    //         route,
    //         tree: HashMap::new(),
    //         search_executed_time: Local::now().to_rfc3339(),
    //         algorithm_runtime: Duration::ZERO,
    //         route_runtime: Duration::ZERO,
    //         search_app_runtime: Duration::ZERO,
    //         iterations: 0,
    //     };
    //     let filename = mock_geometry_file();
    //     let _route_geometry = true;
    //     let _tree_geometry = false;
    //     let geom_plugin =
    //         TraversalPlugin::from_file(&filename, Some(TraversalOutputFormat::Wkt), None).unwrap();

    //     geom_plugin
    //         .process(&mut output_result, &Ok(search_result))
    //         .unwrap();
    //     let geometry_wkt = output_result.get_route_geometry_wkt().unwrap();
    //     assert_eq!(geometry_wkt, expected_geometry);
    // }
}
