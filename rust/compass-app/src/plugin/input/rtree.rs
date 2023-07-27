use compass_core::{map::rtree::VertexRTree, model::property::vertex::Vertex};

use crate::plugin::input::{query::InputQuery, InputPlugin};
use crate::plugin::plugin_error::PluginError;

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

#[cfg(test)]
mod test {
    use serde_json::json;

    use crate::plugin::input::query::InputField;

    use super::*;

    #[test]
    fn test_rtree_plugin() {
        let vertices = vec![
            Vertex::new(0, 0.0, 0.0),
            Vertex::new(1, 1.0, 1.0),
            Vertex::new(2, 2.0, 2.0),
        ];

        let rtree_plugin = build_rtree_plugin(vertices).unwrap();

        let input_query = json!(
            {
                InputField::OriginX.to_str(): 0.1,
                InputField::OriginY.to_str(): 0.1,
                InputField::DestinationX.to_str(): 1.9,
                InputField::DestinationY.to_str(): 2.1,
            }
        );

        let processed_query = rtree_plugin(input_query).unwrap();

        assert_eq!(
            processed_query,
            json!(
                {
                    InputField::OriginX.to_str(): 0.1,
                    InputField::OriginY.to_str(): 0.1,
                    InputField::DestinationX.to_str(): 1.9,
                    InputField::DestinationY.to_str(): 2.1,
                    InputField::OriginVertex.to_str(): 0,
                    InputField::DestinationVertex.to_str(): 2,
                }
            )
        );
    }
}
