use super::edge_rtree_record::EdgeRtreeRecord;
use crate::{
    app::compass::config::compass_configuration_error::CompassConfigurationError,
    plugin::input::input_plugin::InputPlugin,
};
use routee_compass_core::util::{
    fs::{read_decoders, read_utils},
    geo::geo_io_utils::read_linestring_text_file,
    unit::{Distance, DistanceUnit, BASE_DISTANCE_UNIT},
};
use rstar::RTree;
use std::path::Path;

pub struct EdgeRtreeInputPlugin {
    pub rtree: RTree<EdgeRtreeRecord>,
    pub tolerance: Option<(Distance, DistanceUnit)>,
}

impl EdgeRtreeInputPlugin {
    pub fn new(
        road_class_file: &Path,
        linestring_file: &Path,
        tolerance_distance: Option<Distance>,
        distance_unit: Option<DistanceUnit>,
    ) -> Result<Self, CompassConfigurationError> {
        let road_class_lookup: Box<[String]> =
            read_utils::read_raw_file(road_class_file, read_decoders::string, None)?;
        let geometries = read_linestring_text_file(linestring_file)
            .map_err(CompassConfigurationError::IoError)?;

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
            .iter()
            .zip(road_class_lookup.iter())
            .map(|(geom, rc)| EdgeRtreeRecord::new(geom.to_owned(), rc.to_owned()))
            .collect();

        let rtree = RTree::bulk_load(records);

        let tolerance = match (tolerance_distance, distance_unit) {
            (None, None) => None,
            (None, Some(_)) => None,
            (Some(t), None) => Some((t, BASE_DISTANCE_UNIT)),
            (Some(t), Some(u)) => Some((t, u)),
        };

        Ok(EdgeRtreeInputPlugin { rtree, tolerance })
    }
}

impl InputPlugin for EdgeRtreeInputPlugin {
    fn process(
        &self,
        input: &serde_json::Value,
    ) -> Result<Vec<serde_json::Value>, crate::plugin::plugin_error::PluginError> {
        // does the list of valid road classes come from the plugin config, or,
        // from the query?
        // use the nearest_neighbor_iter_with_distance_2 method on rtree, but
        // stop iterating if we exceed the tolerance, if specified
        todo!()
    }
}
