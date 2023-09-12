use std::collections::HashMap;

use super::json_extensions::GeometryJsonExtensions;
use super::utils::{concat_linestrings, parse_linestring};
use crate::app::search::search_app_result::SearchAppResult;
use crate::plugin::output::output_plugin::OutputPlugin;
use crate::plugin::plugin_error::PluginError;
use compass_core::algorithm::search::edge_traversal::EdgeTraversal;
use compass_core::algorithm::search::search_error::SearchError;
use compass_core::algorithm::search::search_tree_branch::SearchTreeBranch;
use compass_core::model::graph::vertex_id::VertexId;
use compass_core::util::fs::fs_utils;
use compass_core::util::fs::read_utils::read_raw_file;
use geo::{LineString, MultiLineString};
use kdam::Bar;
use kdam::BarExt;

pub struct GeometryPlugin {
    geoms: Vec<LineString<f64>>,
    route_geometry: bool,
    tree_geometry: bool,
}

impl GeometryPlugin {
    pub fn from_file(
        filename: &String,
        route_geometry: bool,
        tree_geometry: bool,
    ) -> Result<GeometryPlugin, PluginError> {
        let count = fs_utils::line_count(filename.clone(), fs_utils::is_gzip(&filename))?;

        let mut pb = Bar::builder()
            .total(count)
            .animation("fillup")
            .desc("geometry file")
            .build()
            .map_err(PluginError::InternalError)?;

        let cb = Box::new(|| {
            pb.update(1);
        });
        let geoms = read_raw_file(&filename, parse_linestring, Some(cb))?;
        print!("\n");
        Ok(GeometryPlugin {
            geoms,
            route_geometry,
            tree_geometry,
        })
    }
}

impl OutputPlugin for GeometryPlugin {
    fn proccess(
        &self,
        output: &serde_json::Value,
        search_result: Result<&SearchAppResult, SearchError>,
    ) -> Result<serde_json::Value, PluginError> {
        match search_result {
            Err(_) => Ok(output.clone()),
            Ok(result) => {
                let mut updated_output = output.clone();
                let route_geometry = create_route_geometry(&result.route, &self.geoms)?;
                let tree_geometry = create_tree_geometry(&result.tree, &self.geoms)?;
                updated_output.add_route_geometry(route_geometry)?;
                updated_output.add_tree_geometry(tree_geometry)?;
                Ok(updated_output)
            }
        }
    }
}

fn create_route_geometry(
    route: &Vec<EdgeTraversal>,
    geoms: &Vec<LineString<f64>>,
) -> Result<LineString, PluginError> {
    let edge_ids = route
        .iter()
        .map(|traversal| traversal.edge_id)
        .collect::<Vec<_>>();

    let edge_linestrings = edge_ids
        .iter()
        .map(|eid| {
            let geom = geoms
                .get(eid.0 as usize)
                .ok_or(PluginError::GeometryMissing(eid.0));
            geom
        })
        .collect::<Result<Vec<&LineString>, PluginError>>()?;
    let geometry = concat_linestrings(edge_linestrings);
    return Ok(geometry);
}

fn create_tree_geometry(
    tree: &HashMap<VertexId, SearchTreeBranch>,
    geoms: &Vec<LineString<f64>>,
) -> Result<MultiLineString, PluginError> {
    let edge_ids = tree
        .values()
        .map(|traversal| traversal.edge_traversal.edge_id)
        .collect::<Vec<_>>();

    let tree_linestrings = edge_ids
        .iter()
        .map(|eid| {
            let geom = geoms
                .get(eid.0 as usize)
                .ok_or(PluginError::GeometryMissing(eid.0));
            geom.cloned()
        })
        .collect::<Result<Vec<LineString>, PluginError>>()?;
    let geometry = MultiLineString::new(tree_linestrings);
    return Ok(geometry);
}

#[cfg(test)]
mod tests {
    use compass_core::{
        algorithm::search::edge_traversal::EdgeTraversal,
        model::units::{Length, Ratio},
        model::{
            cost::cost::Cost,
            graph::{edge_id::EdgeId, vertex_id::VertexId},
            property::{edge::Edge, road_class::RoadClass},
            traversal::state::state_variable::StateVar,
        },
        util::fs::read_utils::read_raw_file,
    };
    use geo::{LineString, Point};
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::time::Duration;
    use uom::si;

    use super::*;

    fn mock_edge(edge_id: usize) -> Edge {
        return Edge {
            edge_id: EdgeId(edge_id as u64),
            src_vertex_id: VertexId(0),
            dst_vertex_id: VertexId(1),
            road_class: RoadClass(2),
            distance: Length::new::<si::length::meter>(100.0),
            grade: Ratio::new::<si::ratio::per_mille>(0.0),
        };
    }

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
    fn test_concat_linstrings() {
        let line1 = LineString::from(vec![
            Point::from((0.0, 0.0)),
            Point::from((1.0, 1.0)),
            Point::from((2.0, 2.0)),
        ]);
        let line2 = LineString::from(vec![
            Point::from((3.0, 3.0)),
            Point::from((4.0, 4.0)),
            Point::from((5.0, 5.0)),
        ]);
        let line3 = LineString::from(vec![
            Point::from((6.0, 6.0)),
            Point::from((7.0, 7.0)),
            Point::from((8.0, 8.0)),
        ]);
        let result = concat_linestrings(vec![&line1, &line2, &line3]);
        assert_eq!(result.points().len(), 9);
        let points = result.into_points();
        assert_eq!(points[0], Point::from((0.0, 0.0)));
        assert_eq!(points[8], Point::from((8.0, 8.0)));
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
            search_runtime: Duration::ZERO,
            route_runtime: Duration::ZERO,
            total_runtime: Duration::ZERO,
        };
        let filename = mock_geometry_file().to_str().unwrap().to_string();
        let route_geometry = true;
        let tree_geometry = false;
        let geom_plugin =
            GeometryPlugin::from_file(&filename, route_geometry, tree_geometry).unwrap();

        let result = geom_plugin
            .proccess(&output_result, Ok(&search_result))
            .unwrap();
        let geometry_wkt = result.get_route_geometry_wkt().unwrap();
        assert_eq!(geometry_wkt, expected_geometry);
    }
}
