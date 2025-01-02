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

    /// builds an iterator over map edges ordered by nearness to the given point.
    /// applies the (map-matching) distance tolerance filter.
    pub fn nearest_graph_id_iter<'a>(
        &'a self,
        point: &'a Point<f32>,
    ) -> Box<dyn Iterator<Item = NearestSearchResult> + 'a> {
        match self {
            SpatialIndex::VertexOrientedIndex { rtree, tolerance } => {
                let iter = rtree
                    .nearest_neighbor_iter_with_distance_2(point)
                    .filter(|(obj, _)| obj.test_threshold(point, tolerance).unwrap_or(false))
                    .map(|(next, _)| NearestSearchResult::NearestVertex(next.vertex_id));
                Box::new(iter)
            }
            SpatialIndex::EdgeOrientedIndex { rtree, tolerance } => {
                let iter = rtree
                    .nearest_neighbor_iter_with_distance_2(point)
                    .filter(|(obj, _)| obj.test_threshold(point, tolerance).unwrap_or(false))
                    .map(|(next, _)| NearestSearchResult::NearestEdge(next.edge_id));
                Box::new(iter)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::*;
    use crate::{
        model::network::{Vertex, VertexId},
        util::fs::read_utils,
    };
    use geo;

    #[test]
    fn test_vertex_oriented_e2e() {
        let vertices_filepath = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("model")
            .join("map")
            .join("test")
            .join("rtree_vertices.csv");

        // let query_filepath = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        //     .join("src")
        //     .join("model")
        //     .join("map")
        //     .join("test")
        //     .join("rtree_query.json");

        let vertices: Box<[Vertex]> =
            read_utils::from_csv(&vertices_filepath.as_path(), true, None, None).unwrap();
        let index = SpatialIndex::new_vertex_oriented(&vertices, None);

        // test nearest neighbor queries perform as expected
        let o_result = index
            .nearest_graph_id(&geo::Point(geo::Coord::from((0.101, 0.101))))
            .unwrap();
        let d_result = index
            .nearest_graph_id(&geo::Point(geo::Coord::from((1.901, 2.101))))
            .unwrap();
        match o_result {
            NearestSearchResult::NearestEdge(_) => panic!("should find a vertex!"),
            NearestSearchResult::NearestVertex(vertex_id) => assert_eq!(vertex_id, VertexId(0)),
        }
        match d_result {
            NearestSearchResult::NearestEdge(_) => panic!("should find a vertex!"),
            NearestSearchResult::NearestVertex(vertex_id) => assert_eq!(vertex_id, VertexId(2)),
        }
    }
}
