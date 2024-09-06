use super::{
    map_error::MapError, nearest_search_result::NearestSearchResult,
    vertex_rtree_record::VertexRTreeRecord,
};
use crate::model::{
    property::vertex::Vertex,
    unit::{Distance, DistanceUnit},
};
use geo::Coord;
use rstar::RTree;

pub enum RTreeModel<'a> {
    VertexOriented {
        rtree: RTree<VertexRTreeRecord<'a>>,
        tolerance: Option<(Distance, DistanceUnit)>,
    },
    EdgeOriented,
}

impl<'a> RTreeModel<'a> {
    /// creates a new instance of the rtree model that is vertex-oriented; that is, the
    /// rtree is built over the vertices in the graph, and nearest neighbor searches return
    /// a VertexId.
    pub fn new_vertex_oriented(
        vertices: &'a [Vertex],
        tolerance: Option<(Distance, DistanceUnit)>,
    ) -> Self {
        let rtree_vertices: Vec<VertexRTreeRecord<'a>> =
            vertices.iter().map(VertexRTreeRecord::new).collect();
        let rtree = RTree::bulk_load(rtree_vertices.to_vec());
        let _ = rtree_vertices.get(1);

        Self::VertexOriented { rtree, tolerance }
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
            RTreeModel::EdgeOriented => todo!(),
        }
    }
}
