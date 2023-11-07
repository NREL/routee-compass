use std::path::Path;

use crate::plugin::input::input_json_extensions::InputJsonExtensions;
use crate::plugin::input::input_plugin::InputPlugin;
use crate::plugin::plugin_error::PluginError;
use geo::{coord, Coord};
use routee_compass_core::{
    model::{graph::graph::Graph, property::vertex::Vertex},
    util::fs::read_utils,
};
use rstar::{PointDistance, RTree, RTreeObject, AABB};

pub struct RTreeVertex {
    vertex: Vertex,
}

impl RTreeVertex {
    pub fn new(vertex: Vertex) -> Self {
        Self { vertex }
    }
    pub fn x(&self) -> f64 {
        self.vertex.x()
    }
    pub fn y(&self) -> f64 {
        self.vertex.y()
    }
}

pub struct VertexRTree {
    rtree: RTree<RTreeVertex>,
}

impl VertexRTree {
    pub fn new(vertices: Vec<Vertex>) -> Self {
        let rtree_vertices: Vec<RTreeVertex> = vertices.into_iter().map(RTreeVertex::new).collect();
        let rtree = RTree::bulk_load(rtree_vertices);
        Self { rtree }
    }

    pub fn from_directed_graph(graph: &Graph) -> Self {
        let vertices = graph.vertices.iter().cloned().collect();
        Self::new(vertices)
    }

    pub fn nearest_vertex(&self, point: Coord<f64>) -> Option<&Vertex> {
        match self.rtree.nearest_neighbor(&point) {
            Some(rtree_vertex) => Some(&rtree_vertex.vertex),
            None => None,
        }
    }

    pub fn nearest_vertices(&self, point: Coord<f64>, n: usize) -> Vec<&Vertex> {
        self.rtree
            .nearest_neighbor_iter(&point)
            .take(n)
            .map(|rtv| &rtv.vertex)
            .collect()
    }
}

impl RTreeObject for RTreeVertex {
    type Envelope = AABB<Coord>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_corners(
            coord! {x: self.x(), y: self.y()},
            coord! {x: self.x(), y: self.y()},
        )
    }
}

impl PointDistance for RTreeVertex {
    fn distance_2(&self, point: &Coord) -> f64 {
        let dx = self.x() - point.x;
        let dy = self.y() - point.y;
        dx * dx + dy * dy
    }
}

/// Builds an input plugin that uses an RTree to find the nearest vertex to the origin and destination coordinates.
///
/// # Arguments
///
/// * `vertices` - The vertices to build the RTree from.
///
/// # Returns
///
/// * An input plugin that uses an RTree to find the nearest vertex to the origin and destination coordinates.

pub struct RTreePlugin {
    vertex_rtree: VertexRTree,
}

impl RTreePlugin {
    pub fn new(vertices: Vec<Vertex>) -> Self {
        Self {
            vertex_rtree: VertexRTree::new(vertices),
        }
    }
    pub fn from_file(vertex_file: &Path) -> Result<Self, PluginError> {
        let vertices: Vec<Vertex> = read_utils::vec_from_csv(&vertex_file, true, None, None)
            .map_err(PluginError::CsvReadError)?;
        Ok(Self::new(vertices))
    }
}

impl InputPlugin for RTreePlugin {
    fn process(&self, input: &serde_json::Value) -> Result<Vec<serde_json::Value>, PluginError> {
        let mut updated = input.clone();
        let origin_coord = input.get_origin_coordinate()?;
        let destination_coord = input.get_destination_coordinate()?;

        let origin_vertex = self
            .vertex_rtree
            .nearest_vertex(origin_coord)
            .ok_or(PluginError::NearestVertexNotFound(origin_coord))?;

        let destination_vertex = self
            .vertex_rtree
            .nearest_vertex(destination_coord)
            .ok_or(PluginError::NearestVertexNotFound(destination_coord))?;
        updated.add_origin_vertex(origin_vertex.vertex_id)?;
        updated.add_destination_vertex(destination_vertex.vertex_id)?;
        Ok(vec![updated])
    }
}

#[cfg(test)]
mod test {
    use std::{
        fs::{self},
        path::PathBuf,
    };

    use super::*;
    use crate::plugin::input::input_field::InputField;
    use serde_json::json;

    #[test]
    fn test_rtree_plugin() {
        let vertices_filepath = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("plugin")
            .join("input")
            .join("default")
            .join("test")
            .join("rtree_vertices.csv");

        let query_filepath = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("plugin")
            .join("input")
            .join("default")
            .join("test")
            .join("rtree_query.json");
        let query_str = fs::read_to_string(query_filepath).unwrap();
        let rtree_plugin = RTreePlugin::from_file(&vertices_filepath).unwrap();
        let query: serde_json::Value = serde_json::from_str(&query_str).unwrap();
        let processed_query = rtree_plugin.process(&query).unwrap();

        assert_eq!(
            processed_query[0],
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
