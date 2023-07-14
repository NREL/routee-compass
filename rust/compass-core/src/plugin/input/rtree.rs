use crate::{
    map::{query::UserQuery, rtree::VertexRTree},
    model::property::vertex::Vertex,
    plugin::plugin_error::PluginError,
};
use geo::Coord;

pub fn build_rtree_plugin(
    verticies: Vec<Vertex>,
) -> Result<impl Fn(UserQuery) -> Result<UserQuery, PluginError>, PluginError> {
    let vertex_rtree = VertexRTree::new(verticies);
    let rtree_plugin_fn = move |mut query: UserQuery| -> Result<UserQuery, PluginError> {
        let origin_coord = Coord::from((query.origin_latitude, query.origin_longitude));
        let destination_coord =
            Coord::from((query.destination_latitude, query.destination_longitude));

        let origin_vertex = vertex_rtree
            .nearest_vertex(origin_coord)
            .ok_or(PluginError::NearestVertexNotFound(origin_coord))?;

        let destination_vertex = vertex_rtree
            .nearest_vertex(destination_coord)
            .ok_or(PluginError::NearestVertexNotFound(destination_coord))?;
        query.origin_vertex = Some(origin_vertex.vertex_id);
        query.destination_vertex = Some(destination_vertex.vertex_id);
        Ok(query)
    };
    Ok(rtree_plugin_fn)
}
