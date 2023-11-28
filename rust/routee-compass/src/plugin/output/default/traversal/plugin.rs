use super::json_extensions::TraversalJsonField;
use super::traversal_output_format::TraversalOutputFormat;
use crate::app::compass::compass_app_error::CompassAppError;
use crate::app::search::search_app_result::SearchAppResult;
use crate::plugin::output::output_plugin::OutputPlugin;
use crate::plugin::plugin_error::PluginError;
use geo::LineString;
use kdam::Bar;
use kdam::BarExt;
use routee_compass_core::util::fs::fs_utils;
use routee_compass_core::util::fs::read_utils::read_raw_file;
use routee_compass_core::util::geo::geo_io_utils;
use std::path::Path;

pub struct TraversalPlugin {
    geoms: Box<[LineString<f64>]>,
    route: Option<TraversalOutputFormat>,
    tree: Option<TraversalOutputFormat>,
}

impl TraversalPlugin {
    pub fn from_file<P: AsRef<Path>>(
        filename: &P,
        route: Option<TraversalOutputFormat>,
        tree: Option<TraversalOutputFormat>,
    ) -> Result<TraversalPlugin, PluginError> {
        let count =
            fs_utils::line_count(filename.clone(), fs_utils::is_gzip(filename)).map_err(|e| {
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
        Ok(TraversalPlugin { geoms, route, tree })
    }
}

impl OutputPlugin for TraversalPlugin {
    fn process(
        &self,
        output: &serde_json::Value,
        search_result: &Result<SearchAppResult, CompassAppError>,
    ) -> Result<Vec<serde_json::Value>, PluginError> {
        match search_result {
            Err(_) => Ok(vec![output.clone()]),
            Ok(result) => {
                let mut output_mut = output.clone();
                let updated = output_mut
                    .as_object_mut()
                    .ok_or(PluginError::InternalError(format!(
                        "expected output JSON to be an object, found {}",
                        output
                    )))?;

                match self.route {
                    None => {}
                    Some(route_args) => {
                        let route_output =
                            route_args.generate_route_output(&result.route, &self.geoms)?;
                        updated.insert(TraversalJsonField::RouteOutput.to_string(), route_output);
                    }
                }

                match self.tree {
                    None => {}
                    Some(tree_args) => {
                        let route_output =
                            tree_args.generate_tree_output(&result.tree, &self.geoms)?;
                        updated.insert(TraversalJsonField::TreeOutput.to_string(), route_output);
                    }
                }

                Ok(vec![serde_json::Value::Object(updated.to_owned())])
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::output::default::traversal::json_extensions::TraversalJsonExtensions;
    use chrono::Local;
    use routee_compass_core::{
        algorithm::search::edge_traversal::EdgeTraversal,
        model::{
            cost::Cost, road_network::edge_id::EdgeId, traversal::state::state_variable::StateVar,
        },
        util::{fs::read_utils::read_raw_file, geo::geo_io_utils::parse_linestring},
    };
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::time::Duration;

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
        let result = read_raw_file(&mock_geometry_file(), parse_linestring, None).unwrap();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_add_geometry() {
        let expected_geometry = String::from("LINESTRING(0 0,1 1,2 2,3 3,4 4,5 5,6 6,7 7,8 8)");
        let output_result = serde_json::json!({});
        let route = vec![
            EdgeTraversal {
                edge_id: EdgeId(0),
                access_cost: Cost::from(0.0),
                traversal_cost: Cost::from(0.0),
                result_state: vec![StateVar(0.0)],
            },
            EdgeTraversal {
                edge_id: EdgeId(1),
                access_cost: Cost::from(0.0),
                traversal_cost: Cost::from(0.0),
                result_state: vec![StateVar(0.0)],
            },
            EdgeTraversal {
                edge_id: EdgeId(2),
                access_cost: Cost::from(0.0),
                traversal_cost: Cost::from(0.0),
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
        let filename = mock_geometry_file();
        let _route_geometry = true;
        let _tree_geometry = false;
        let geom_plugin =
            TraversalPlugin::from_file(&filename, Some(TraversalOutputFormat::Wkt), None).unwrap();

        let result = geom_plugin
            .process(&output_result, &Ok(search_result))
            .unwrap();
        let geometry_wkt = result[0].get_route_geometry_wkt().unwrap();
        assert_eq!(geometry_wkt, expected_geometry);
    }
}
