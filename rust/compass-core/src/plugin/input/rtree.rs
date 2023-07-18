use crate::{
    map::rtree::VertexRTree, model::property::vertex::Vertex, plugin::plugin_error::PluginError,
};
use super::query::InputQuery;

pub fn build_rtree_plugin(
    verticies: Vec<Vertex>,
) -> Result<impl Fn(serde_json::Value) -> Result<serde_json::Value, PluginError>, PluginError> {
    let vertex_rtree = VertexRTree::new(verticies);
    let rtree_plugin_fn =
        move |mut query: serde_json::Value| -> Result<serde_json::Value, PluginError> {
            let origin_coord = query.get_origin_coordinate()?;
            let destination_coord = query.get_destination_coordinate()?;

            let origin_vertex = vertex_rtree
                .nearest_vertex(origin_coord)
                .ok_or(PluginError::NearestVertexNotFound(origin_coord))?;

            let destination_vertex = vertex_rtree
                .nearest_vertex(destination_coord)
                .ok_or(PluginError::NearestVertexNotFound(destination_coord))?;
            query.add_origin_vertex(origin_vertex.vertex_id)?;
            query.add_destination_vertex(destination_vertex.vertex_id)?;
            Ok(query)
        };
    Ok(rtree_plugin_fn)
}
