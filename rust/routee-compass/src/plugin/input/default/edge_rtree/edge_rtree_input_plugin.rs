use super::edge_rtree_record::EdgeRtreeRecord;
use crate::{
    app::compass::config::{
        compass_configuration_error::CompassConfigurationError,
        frontier_model::{
            road_class::road_class_parser::RoadClassParser,
            vehicle_restrictions::{
                vehicle_parameters::VehicleParameters, vehicle_restriction::VehicleRestriction,
                vehicle_restriction_builder::vehicle_restriction_lookup_from_file,
            },
        },
    },
    plugin::input::{
        input_json_extensions::InputJsonExtensions, input_plugin::InputPlugin, InputPluginError,
    },
};
use geo_types::Coord;
use routee_compass_core::{
    model::network::edge_id::EdgeId,
    model::unit::{as_f64::AsF64, Distance, DistanceUnit, BASE_DISTANCE_UNIT},
    util::{
        fs::{read_decoders, read_utils},
        geo::geo_io_utils::read_linestring_text_file,
    },
};
use rstar::RTree;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

pub struct EdgeRtreeInputPlugin {
    pub rtree: RTree<EdgeRtreeRecord>,
    pub tolerance: Option<(Distance, DistanceUnit)>,

    // TODO: instead of having to load the road classes and the truck restrictions
    // it would be cleaner to bring in the FrontierModel into scope so we can just
    // validate an edge based on the whatever the frontier model is.

    // Road class lookup table in case some road classes are restricted
    pub road_class_lookup: Option<Vec<u8>>,
    pub road_class_parser: RoadClassParser,

    // Vehicle restrictions
    pub vehicle_restrictions: Option<HashMap<EdgeId, Vec<VehicleRestriction>>>,
}

impl InputPlugin for EdgeRtreeInputPlugin {
    /// finds the nearest edge ids to the user-provided origin and destination coordinates.
    /// optionally restricts the search to a subset of road classes tagged by the user.
    fn process(&self, query: &mut serde_json::Value) -> Result<(), InputPluginError> {
        let road_classes = self.road_class_parser.read_query(query).map_err(|e| {
            InputPluginError::InputPluginFailed(format!(
                "Unable to apply EdgeRtree Input Plugin due to:\n\n{}",
                e
            ))
        })?;
        let vehicle_parameters = VehicleParameters::from_query(query).ok();

        let src_coord = query.get_origin_coordinate()?;
        let dst_coord_option = query.get_destination_coordinate()?;

        let source_edge_id = search(
            src_coord,
            &self.rtree,
            self.tolerance,
            &self.road_class_lookup,
            &road_classes,
            &self.vehicle_restrictions,
            &vehicle_parameters,
        )?
        .ok_or_else(|| matching_error(&src_coord, self.tolerance))?;
        let destination_edge_id_option = match dst_coord_option {
            None => Ok(None),
            Some(dst_coord) => search(
                dst_coord,
                &self.rtree,
                self.tolerance,
                &self.road_class_lookup,
                &road_classes,
                &self.vehicle_restrictions,
                &vehicle_parameters,
            )?
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
        road_class_file: Option<String>,
        vehicle_restriction_file: Option<String>,
        linestring_file: String,
        tolerance_distance: Option<Distance>,
        distance_unit: Option<DistanceUnit>,
        road_class_parser: RoadClassParser,
    ) -> Result<Self, CompassConfigurationError> {
        let road_class_lookup: Option<Vec<u8>> = match road_class_file {
            None => Ok(None),
            Some(file) => read_utils::read_raw_file(file, read_decoders::u8, None)
                .map(|r| Some(r.into_vec()))
                .map_err(CompassConfigurationError::IoError),
        }?;
        let vehicle_restrictions: Option<HashMap<EdgeId, Vec<VehicleRestriction>>> =
            match vehicle_restriction_file {
                None => None,
                Some(file) => {
                    let path = PathBuf::from(file);
                    let trs = vehicle_restriction_lookup_from_file(&path)?;
                    Some(trs)
                }
            };

        let geometries = read_linestring_text_file(linestring_file)
            .map_err(CompassConfigurationError::IoError)?
            .into_vec();

        let rcl_len_opt = road_class_lookup.as_ref().map(|l| l.len());
        let geo_len = geometries.len();
        if let Some(rcl_len) = rcl_len_opt {
            if rcl_len != geo_len {
                let msg = format!(
                    "edge_rtree: road class file and geometries file have different lengths ({} != {})",
                    rcl_len, geo_len
                );
                return Err(CompassConfigurationError::UserConfigurationError(msg));
            }
        }

        let records: Vec<EdgeRtreeRecord> = geometries
            .into_iter()
            .enumerate()
            .map(|(idx, geom)| EdgeRtreeRecord::new(EdgeId(idx), geom))
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
            road_class_lookup,
            tolerance,
            road_class_parser,
            vehicle_restrictions,
        })
    }
}

/// finds the nearest edge to some coordinate, optionally within some distance tolerance
///
/// # Arguments
///
/// * `coord` - coordinate from which to find a nearest edge
/// * `rtree` - search tree containing all road network edges
/// * `tolerance` - distance tolerance argument. if provided, result edge must be within this
///                 distance/distance unit of the coord provided.
/// * `road_class_lookup` - optional lookup table for road classes
/// * `road_classes` - optional set of road classes to restrict search to
/// * `vehicle_restrictions` - optional lookup table for truck restrictions
/// * `vehicle_parameters` - truck parameters to validate against truck restrictions
///
/// # Result
///
/// the EdgeId of the nearest edge that meets the tolerance requirement, if provided
fn search(
    coord: Coord<f32>,
    rtree: &RTree<EdgeRtreeRecord>,
    tolerance: Option<(Distance, DistanceUnit)>,
    road_class_lookup: &Option<Vec<u8>>,
    road_classes: &Option<HashSet<u8>>,
    vehicle_restrictions: &Option<HashMap<EdgeId, Vec<VehicleRestriction>>>,
    vehicle_parameters: &Option<VehicleParameters>,
) -> Result<Option<EdgeId>, InputPluginError> {
    let point = geo::Point(coord);
    for (record, distance_meters) in rtree.nearest_neighbor_iter_with_distance_2(&point) {
        if !within_tolerance(tolerance, &distance_meters) {
            return Ok(None);
        }
        let valid_class = match (road_classes, road_class_lookup) {
            (Some(valid_classes), Some(lookup)) => {
                let this_class = lookup.get(record.edge_id.0).ok_or_else(|| {
                    InputPluginError::InputPluginFailed(format!(
                        "edge rtree road class file missing edge {}",
                        record.edge_id
                    ))
                })?;
                valid_classes.contains(this_class)
            }
            _ => true,
        };
        let valid_truck = match (vehicle_restrictions, vehicle_parameters) {
            (Some(vehicle_restrictions), Some(vehicle_parameters)) => {
                let restrictions = vehicle_restrictions.get(&record.edge_id);
                if let Some(restrictions) = restrictions {
                    restrictions
                        .iter()
                        .all(|restriction| restriction.valid(vehicle_parameters))
                } else {
                    true
                }
            }
            _ => true,
        };
        if valid_class && valid_truck {
            return Ok(Some(record.edge_id));
        }
    }
    Ok(None)
}

/// helper to build a matching error response
fn matching_error(
    coord: &Coord<f32>,
    tolerance: Option<(Distance, DistanceUnit)>,
) -> InputPluginError {
    let mut message = format!("unable to match coordinate {:?} to network edge", coord);
    if let Some((dist, unit)) = tolerance {
        message.push_str(&format!(" within tolerance of {} {}", dist, unit))
    };
    InputPluginError::InputPluginFailed(message)
}

/// helper to test if some distance in meters is within the optionally-provided tolerance
fn within_tolerance(tolerance: Option<(Distance, DistanceUnit)>, distance_meters: &f32) -> bool {
    match tolerance {
        None => true,
        Some((tolerance, distance_unit)) => {
            let tolerance_meters = distance_unit
                .convert(&tolerance, &DistanceUnit::Meters)
                .as_f64() as f32;

            distance_meters <= &tolerance_meters
        }
    }
}
