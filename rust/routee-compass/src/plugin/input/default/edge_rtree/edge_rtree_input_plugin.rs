use super::edge_rtree_record::EdgeRtreeRecord;
use crate::{
    app::compass::config::{
        compass_configuration_error::CompassConfigurationError,
        config_json_extension::ConfigJsonExtensions,
        frontier_model::road_class::road_class_parser::RoadClassParser,
    },
    plugin::{
        input::{input_json_extensions::InputJsonExtensions, input_plugin::InputPlugin},
        plugin_error::PluginError,
    },
};
use geo_types::Coord;
use routee_compass_core::{
    model::road_network::edge_id::EdgeId,
    model::unit::{as_f64::AsF64, Distance, DistanceUnit, BASE_DISTANCE_UNIT},
    util::{
        fs::{read_decoders, read_utils},
        geo::geo_io_utils::read_linestring_text_file,
    },
};
use rstar::RTree;
use std::{collections::HashSet, path::Path};

pub struct EdgeRtreeInputPlugin {
    pub rtree: RTree<EdgeRtreeRecord>,
    pub tolerance: Option<(Distance, DistanceUnit)>,
    pub road_class_parser: RoadClassParser,
}

impl InputPlugin for EdgeRtreeInputPlugin {
    /// finds the nearest edge ids to the user-provided origin and destination coordinates.
    /// optionally restricts the search to a subset of road classes tagged by the user.
    fn process(&self, query: &mut serde_json::Value) -> Result<(), PluginError> {
        let road_classes = self.road_class_parser.read_query(query).map_err(|e| {
            PluginError::InputError(format!(
                "Unable to process EdgeRtree Input Plugin due to: {}",
                e
            ))
        })?;
        let src_coord = query.get_origin_coordinate()?;
        let dst_coord_option = query.get_destination_coordinate()?;

        let source_edge_id = search(&self.rtree, src_coord, &road_classes, self.tolerance)
            .ok_or_else(|| matching_error(&src_coord, self.tolerance))?;
        let destination_edge_id_option = match dst_coord_option {
            None => Ok(None),
            Some(dst_coord) => search(&self.rtree, dst_coord, &road_classes, self.tolerance)
                .map(Some)
                .ok_or_else(|| matching_error(&dst_coord, self.tolerance)),
        }?;

        query.add_origin_edge(source_edge_id)?;
        match destination_edge_id_option {
            None => {}
            Some(destination_edge_id) => {
                query.add_destination_edge(destination_edge_id)?;
            }
        }

        Ok(())
    }
}

impl EdgeRtreeInputPlugin {
    pub fn new(
        road_class_file: &Path,
        linestring_file: &Path,
        tolerance_distance: Option<Distance>,
        distance_unit: Option<DistanceUnit>,
        road_class_parser: RoadClassParser,
    ) -> Result<Self, CompassConfigurationError> {
        let road_class_lookup: Vec<u8> =
            read_utils::read_raw_file(road_class_file, read_decoders::u8, None)?.into_vec();
        let geometries = read_linestring_text_file(linestring_file)
            .map_err(CompassConfigurationError::IoError)?
            .into_vec();

        let rcl_len = road_class_lookup.len();
        let geo_len = geometries.len();
        if rcl_len != geo_len {
            let msg = format!(
                "edge_rtree: road class file and geometries file have different lengths ({} != {})",
                rcl_len, geo_len
            );
            return Err(CompassConfigurationError::UserConfigurationError(msg));
        }

        let records: Vec<EdgeRtreeRecord> = geometries
            .into_iter()
            .enumerate()
            .zip(road_class_lookup)
            .map(|((idx, geom), rc)| EdgeRtreeRecord::new(EdgeId(idx), geom, rc))
            .collect();

        let rtree = RTree::bulk_load(records);

        let tolerance = match (tolerance_distance, distance_unit) {
            (None, None) => None,
            (None, Some(_)) => None,
            (Some(t), None) => Some((t, BASE_DISTANCE_UNIT)),
            (Some(t), Some(u)) => Some((t, u)),
        };

        Ok(EdgeRtreeInputPlugin {
            rtree,
            tolerance,
            road_class_parser,
        })
    }
}

/// finds the nearest edge to some coordinate, optionally within some distance tolerance
///
/// # Arguments
///
/// * `rtree` - search tree containing all road network edges
/// * `coord` - coordinate from which to find a nearest edge
/// * `tolerance` - distance tolerance argument. if provided, result edge must be within this
///                 distance/distance unit of the coord provided.
///
/// # Result
///
/// the EdgeId of the nearest edge that meets the tolerance requirement, if provided
fn search(
    rtree: &RTree<EdgeRtreeRecord>,
    coord: Coord<f32>,
    road_classes: &Option<HashSet<u8>>,
    tolerance: Option<(Distance, DistanceUnit)>,
) -> Option<EdgeId> {
    let point = geo::Point(coord);
    let search_result = rtree
        .nearest_neighbor_iter_with_distance_2(&point)
        .find(|(record, distance_meters)| {
            let within_distance = within_tolerance(tolerance, distance_meters);
            let valid_class = match &road_classes {
                None => true,
                Some(valid_classes) => valid_classes.contains(&record.road_class),
            };
            within_distance && valid_class
        })
        .map(|(record, _dist)| record.edge_id.to_owned());
    search_result
}

/// helper to build a matching error response
fn matching_error(coord: &Coord<f32>, tolerance: Option<(Distance, DistanceUnit)>) -> PluginError {
    let mut message = format!("unable to match coordinate {:?} to network edge", coord);
    if let Some((dist, unit)) = tolerance {
        message.push_str(&format!(" within tolerance of {} {}", dist, unit))
    };
    PluginError::PluginFailed(message)
}

/// helper to test if some distance in meters is within the optionally-provided tolerance
fn within_tolerance(tolerance: Option<(Distance, DistanceUnit)>, distance_meters: &f32) -> bool {
    match tolerance {
        None => true,
        Some((tolerance, distance_unit)) => {
            let tolerance_meters = distance_unit
                .convert(tolerance, DistanceUnit::Meters)
                .as_f64() as f32;

            distance_meters <= &tolerance_meters
        }
    }
}
