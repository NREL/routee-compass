use super::json_extensions::GeometryJsonExtensions;
use super::utils::{concat_linestrings, parse_linestring};
use crate::plugin::output::OutputPlugin;
use crate::plugin::plugin_error::PluginError;
use compass_core::algorithm::search::edge_traversal::EdgeTraversal;
use compass_core::algorithm::search::search_error::SearchError;
use compass_core::util::fs::fs_utils;
use compass_core::util::fs::read_utils::read_raw_file;
use geo::LineString;
use kdam::Bar;
use kdam::BarExt;

/// Build a geometry plugin from a file containing a list of linestrings where each row
/// index represents the edge id of the linestring.
pub fn build_geometry_plugin_from_file(filename: &String) -> Result<OutputPlugin, PluginError> {
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
    let geometry_lookup_fn = move |output: &serde_json::Value,
                                   search_result: Result<&Vec<EdgeTraversal>, SearchError>|
          -> Result<serde_json::Value, PluginError> {
        let mut updated_output = output.clone();
        let edge_ids = search_result?
            .iter()
            .map(|traversal| traversal.edge_id)
            .collect::<Vec<_>>();

        let final_linestring = edge_ids
            .iter()
            .map(|eid| {
                let geom = geoms
                    .get(eid.0 as usize)
                    .ok_or(PluginError::GeometryMissing(eid.0));
                geom
            })
            .collect::<Result<Vec<&LineString>, PluginError>>()?;
        let geometry = concat_linestrings(final_linestring);
        updated_output.add_geometry(geometry)?;
        Ok(updated_output)
    };
    Ok(Box::new(geometry_lookup_fn))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use compass_core::{
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
        let filename = mock_geometry_file().to_str().unwrap().to_string();
        let geom_plugin = build_geometry_plugin_from_file(&filename).unwrap();

        let result = geom_plugin(&output_result, Ok(&route)).unwrap();
        let geometry_wkt = result.get_geometry_wkt().unwrap();
        assert_eq!(geometry_wkt, expected_geometry);
    }
}
