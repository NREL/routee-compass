use super::{
    edge_rtree_record::EdgeRtreeRecord, map_error::MapError,
    nearest_search_result::NearestSearchResult, vertex_rtree_record::VertexRtreeRecord,
};
use crate::model::{
    property::{edge::Edge, vertex::Vertex},
    unit::{Distance, DistanceUnit},
};
use geo::{Coord, LineString};
use rstar::RTree;

pub enum RTreeModel<'a> {
    VertexOriented {
        rtree: RTree<VertexRtreeRecord<'a>>,
        tolerance: Option<(Distance, DistanceUnit)>,
    },
    EdgeOriented {
        rtree: RTree<EdgeRtreeRecord<'a>>,
        tolerance: Option<(Distance, DistanceUnit)>,
    },
}

impl<'a> RTreeModel<'a> {
    /// creates a new instance of the rtree model that is vertex-oriented; that is, the
    /// rtree is built over the vertices in the graph, and nearest neighbor searches return
    /// a VertexId.
    pub fn new_vertex_oriented(
        vertices: &'a [Vertex],
        tolerance: Option<(Distance, DistanceUnit)>,
    ) -> Self {
        let rtree_vertices: Vec<VertexRtreeRecord<'a>> =
            vertices.iter().map(VertexRtreeRecord::new).collect();
        let rtree = RTree::bulk_load(rtree_vertices.to_vec());

        Self::VertexOriented { rtree, tolerance }
    }

    /// creates a new instance of the rtree model that is edge-oriented; that is, the
    /// rtree is built over the edges in the graph, and nearest neighbor searches return
    /// the edge's destination vertex.
    /// - future work: make SearchOrientation set which incident vertex is returned.
    pub fn new_edge_oriented(
        edges: &'a [Edge],
        edge_geometries: &'a [LineString<f32>],
        tolerance: Option<(Distance, DistanceUnit)>,
    ) -> Self {
        let rtree_edges: Vec<EdgeRtreeRecord<'a>> = edges
            .iter()
            .zip(edge_geometries.iter())
            .map(|(e, g)| EdgeRtreeRecord::new(e, g))
            .collect();
        let rtree = RTree::bulk_load(rtree_edges.to_vec());

        Self::EdgeOriented { rtree, tolerance }
    }

    /// gets the nearest graph id, which is a VertexId or EdgeId depending on the orientation
    /// of the RTree.
    pub fn nearest_graph_id(&self, coord: &Coord<f32>) -> Result<NearestSearchResult, MapError> {
        match self {
            RTreeModel::VertexOriented { rtree, tolerance } => {
                let nearest = rtree.nearest_neighbor(coord).ok_or_else(|| {
                    MapError::MapMatchError(String::from("no map vertices exist for matching"))
                })?;
                nearest.within_distance_threshold(coord, tolerance)?;
                Ok(NearestSearchResult::NearestVertex(nearest.vertex.vertex_id))
            }
            RTreeModel::EdgeOriented { rtree, tolerance } => {
                let nearest = rtree.nearest_neighbor(&geo::Point(*coord)).ok_or_else(|| {
                    MapError::MapMatchError(String::from("no map vertices exist for matching"))
                })?;
                nearest.within_distance_threshold(coord, tolerance)?;
                Ok(NearestSearchResult::NearestEdge(nearest.edge.edge_id))
            }
        }
    }
}
