use super::{
    map_error::MapError, map_matching_result::MapMatchingResult,
    vertex_rtree_record::VertexRTreeRecord,
};
use crate::model::{property::vertex::Vertex, unit::Distance};
use geo::Coord;
use rstar::RTree;

pub enum RTreeModel<'a> {
    VertexOriented {
        rtree: RTree<VertexRTreeRecord<'a>>,
        tolerance: Distance,
    },
    EdgeOriented,
}

impl<'a> RTreeModel<'a> {
    pub fn new_vertex_oriented(vertices: &'a [Vertex], tolerance: Distance) -> Self {
        let rtree_vertices: Vec<VertexRTreeRecord<'a>> =
            vertices.iter().map(VertexRTreeRecord::new).collect();
        let rtree = RTree::bulk_load(rtree_vertices.to_vec());
        let _ = rtree_vertices.get(1);

        Self::VertexOriented { rtree, tolerance }
    }

    pub fn map_match(&self, query: &mut serde_json::Value) -> Result<MapMatchingResult, MapError> {
        match self {
            RTreeModel::VertexOriented { rtree, tolerance } => {
                // let src_coord = query.get_origin_coordinate()?;
                // let dst_coord_option = query.get_destination_coordinate()?;
                // rtree
                //     .nearest_neighbor(&point)
                //     .map(|v| v.vertex)
                //     .ok_or_else(|| MapError::MapMatchError(format!("")))
                todo!()
            }
            RTreeModel::EdgeOriented => todo!(),
        }
        todo!()
    }
}
