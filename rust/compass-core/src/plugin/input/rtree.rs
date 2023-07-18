use super::{query::InputQuery, InputPlugin};
use crate::{
    map::rtree::VertexRTree, model::property::vertex::Vertex, plugin::plugin_error::PluginError,
};

/// Builds an input plugin that uses an RTree to find the nearest vertex to the origin and destination coordinates.
/// 
/// # Arguments
/// 
/// * `vertices` - The vertices to build the RTree from.
/// 
/// # Returns
/// 
/// * An input plugin that uses an RTree to find the nearest vertex to the origin and destination coordinates.
pub fn build_rtree_plugin(vertices: Vec<Vertex>) -> Result<InputPlugin, PluginError> {
    let vertex_rtree = VertexRTree::new(vertices);
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
    Ok(Box::new(rtree_plugin_fn))
}
