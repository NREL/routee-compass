use super::map_error::MapError;
use super::rtree_model::RTreeModel;
use crate::model::map::map_json_extensions::MapJsonExtensions;
use crate::model::map::nearest_search_result::NearestSearchResult;
use std::sync::Arc;

pub struct MapModel<'a> {
    rtree_model: Arc<RTreeModel<'a>>,
}

impl<'a> MapModel<'a> {
    pub fn map_match(&self, query: &mut serde_json::Value) -> Result<(), MapError> {
        let src_coord = query.get_origin_coordinate()?;
        match self.rtree_model.nearest_graph_id(&src_coord)? {
            NearestSearchResult::NearestVertex(vertex_id) => {
                query.add_origin_vertex(vertex_id)?;
            }
            NearestSearchResult::NearestEdge(edge_id) => query.add_origin_edge(edge_id)?,
        }

        let dst_coord_option = query.get_destination_coordinate()?;
        match dst_coord_option {
            None => {}
            Some(dst_coord) => match self.rtree_model.nearest_graph_id(&dst_coord)? {
                NearestSearchResult::NearestVertex(vertex_id) => {
                    query.add_destination_vertex(vertex_id)?;
                }
                NearestSearchResult::NearestEdge(edge_id) => query.add_destination_edge(edge_id)?,
            },
        }

        Ok(())
    }
}
