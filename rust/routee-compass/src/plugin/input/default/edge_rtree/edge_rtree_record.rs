use geo::{Centroid, LineString, Point};
use routee_compass_core::{
    model::road_network::edge_id::EdgeId,
    util::{
        geo::haversine,
        unit::{as_f64::AsF64, Distance},
    },
};
use rstar::{PointDistance, RTreeObject, AABB};

pub struct EdgeRtreeRecord {
    pub edge_id: EdgeId,
    pub geometry: LineString,
    pub road_class: String,
}

impl EdgeRtreeRecord {
    pub fn new(edge_id: EdgeId, geometry: LineString, road_class: String) -> EdgeRtreeRecord {
        EdgeRtreeRecord {
            edge_id,
            geometry,
            road_class,
        }
    }
}

impl RTreeObject for EdgeRtreeRecord {
    type Envelope = AABB<Point>;
    fn envelope(&self) -> Self::Envelope {
        self.geometry.envelope()
    }
}

impl PointDistance for EdgeRtreeRecord {
    /// compares query nearness via the "centroid" of this LineString,
    /// the midpoint of the bounding box of the line.
    ///
    /// # Arguments
    ///
    /// * `point` - point query of a nearest neighbors search
    ///
    /// # Returns
    ///
    /// * distance in meters (assumes points are in WGS84)
    fn distance_2(&self, point: &Point) -> f64 {
        let this_point = self
            .geometry
            .centroid()
            .unwrap_or_else(|| panic!("empty linestring in geometry file"));
        let distance = haversine::coord_distance_meters(this_point.0, point.0)
            .unwrap_or(Distance::new(f64::MAX));
        distance.as_f64()
    }
}
