use std::sync::Arc;

use super::{
    geometry_model::GeometryModel, map_edge_rtree_object::MapEdgeRTreeObject, map_error::MapError,
    map_vertex_rtree_object::MapVertexRTreeObject, nearest_search_result::NearestSearchResult,
};
use crate::model::{
    network::{Graph, Vertex},
    unit::{Distance, DistanceUnit},
};
use geo::Point;
use rstar::RTree;

pub enum SpatialIndex {
    VertexOrientedIndex {
        rtree: RTree<MapVertexRTreeObject>,
        tolerance: Option<(Distance, DistanceUnit)>,
    },
    EdgeOrientedIndex {
        rtree: RTree<MapEdgeRTreeObject>,
        tolerance: Option<(Distance, DistanceUnit)>,
    },
}

impl SpatialIndex {
    /// creates a new instance of the rtree model that is vertex-oriented; that is, the
    /// rtree is built over the vertices in the graph, and nearest neighbor searches return
    /// a VertexId.
    pub fn new_vertex_oriented(
        vertices: &[Vertex],
        tolerance: Option<(Distance, DistanceUnit)>,
    ) -> Self {
        let entries: Vec<MapVertexRTreeObject> =
            vertices.iter().map(MapVertexRTreeObject::new).collect();
        let rtree = RTree::bulk_load(entries);
        Self::VertexOrientedIndex { rtree, tolerance }
    }

    /// creates a new instance of the rtree model that is edge-oriented; that is, the
    /// rtree is built over the edges in the graph, and nearest neighbor searches return
    /// the edge's destination vertex.
    /// - future work: make SearchOrientation set which incident vertex is returned.
    pub fn new_edge_oriented(
        graph: Arc<Graph>,
        geometry_model: &GeometryModel,
        tolerance: Option<(Distance, DistanceUnit)>,
    ) -> Self {
        let entries: Vec<MapEdgeRTreeObject> = graph
            .edges
            .iter()
            .zip(geometry_model.geometries())
            .map(|(e, g)| MapEdgeRTreeObject::new(e, g))
            .collect();
        let rtree = RTree::bulk_load(entries.to_vec());

        Self::EdgeOrientedIndex { rtree, tolerance }
    }

    /// gets the nearest graph id, which is a VertexId or EdgeId depending on the orientation
    /// of the RTree.
    pub fn nearest_graph_id(&self, point: &Point<f32>) -> Result<NearestSearchResult, MapError> {
        match self {
            SpatialIndex::VertexOrientedIndex { rtree, tolerance } => {
                let nearest = rtree.nearest_neighbor(point).ok_or_else(|| {
                    MapError::MapMatchError(String::from("no map vertices exist for matching"))
                })?;
                nearest.within_distance_threshold(point, tolerance)?;
                Ok(NearestSearchResult::NearestVertex(nearest.vertex_id))
            }
            SpatialIndex::EdgeOrientedIndex { rtree, tolerance } => {
                let nearest = rtree.nearest_neighbor(point).ok_or_else(|| {
                    MapError::MapMatchError(String::from("no map vertices exist for matching"))
                })?;
                nearest.within_distance_threshold(point, tolerance)?;
                Ok(NearestSearchResult::NearestEdge(nearest.edge_id))
            }
        }
    }
}
