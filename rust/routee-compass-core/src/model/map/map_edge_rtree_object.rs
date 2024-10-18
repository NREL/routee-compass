use super::{
    geometry_model::GeometryModel, map_error::MapError, nearest_search_result::NearestSearchResult,
    spatial_index_ops::within_threshold,
};
use crate::{
    model::{
        property::{edge::Edge, vertex::Vertex},
        road_network::{edge_id::EdgeId, vertex_id::VertexId},
        unit::{Distance, DistanceUnit},
    },
    util::geo::haversine,
};
use geo::{coord, LineString, Point};
use rstar::{PointDistance, RTree, RTreeObject, AABB};

impl MapEdgeRTreeObject {
    pub fn new(edge: &Edge, linestring: &LineString<f32>) -> MapEdgeRTreeObject {
        MapEdgeRTreeObject {
            edge_id: edge.edge_id,
            envelope: linestring.envelope(),
        }
    }

    pub fn within_distance_threshold(
        &self,
        point: &Point<f32>,
        tolerance: &Option<(Distance, DistanceUnit)>,
    ) -> Result<(), MapError> {
        match tolerance {
            Some((dist, unit)) => within_threshold(&self.envelope, point, *dist, *unit),
            None => Ok(()),
        }
    }
}

#[derive(Clone)]
pub struct MapEdgeRTreeObject {
    pub edge_id: EdgeId,
    pub envelope: AABB<Point<f32>>,
}

impl RTreeObject for MapEdgeRTreeObject {
    type Envelope = AABB<Point<f32>>;

    fn envelope(&self) -> Self::Envelope {
        self.envelope
    }
}

impl PointDistance for MapEdgeRTreeObject {
    fn distance_2(&self, point: &Point<f32>) -> f32 {
        self.envelope.distance_2(point)
    }
}
