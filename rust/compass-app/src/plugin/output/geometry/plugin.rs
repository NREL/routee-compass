use compass_core::util::fs::read_utils::read_raw_file;
use geo::LineString;

use crate::plugin::output::OutputPlugin;
use crate::plugin::plugin_error::PluginError;

use super::json_extensions::GeometryJsonExtensions;
use super::utils::{concat_linestrings, parse_linestring};

/// Build a geometry plugin from a file containing a list of linestrings where each row
/// index represents the edge id of the linestring.
pub fn build_geometry_plugin_from_file(filename: String) -> Result<OutputPlugin, PluginError> {
    let geoms = read_raw_file(&filename, parse_linestring)?;
    let geometry_lookup_fn =
        move |mut output: serde_json::Value| -> Result<serde_json::Value, PluginError> {
            let edge_ids = output.get_edge_ids()?;
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
            output.add_geometry(geometry)?;
            Ok(output)
        };
    Ok(Box::new(geometry_lookup_fn))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use compass_core::util::fs::read_utils::read_raw_file;
    use geo::{LineString, Point};

    use crate::plugin::output::geometry::json_extensions::GeometryJsonField;

    use super::*;

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
        let result = read_raw_file(&mock_geometry_file(), parse_linestring).unwrap();
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
        let output_result = serde_json::json!(
            {
                "path": [
                    {
                        GeometryJsonField::EdgeId.as_str(): 0,
                    },
                    {
                        GeometryJsonField::EdgeId.as_str(): 1,
                    },
                    {
                        GeometryJsonField::EdgeId.as_str(): 2,
                    }
                ]
            }
        );
        let geom_plugin =
            build_geometry_plugin_from_file(mock_geometry_file().to_str().unwrap().to_string())
                .unwrap();

        let result = geom_plugin(output_result).unwrap();
        let geometry_wkt = result.get_geometry_wkt().unwrap();
        assert_eq!(geometry_wkt, expected_geometry);
    }
}
