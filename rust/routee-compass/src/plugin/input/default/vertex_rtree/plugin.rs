use std::path::Path;

use crate::plugin::input::input_json_extensions::InputJsonExtensions;
use crate::plugin::input::input_plugin::InputPlugin;
use crate::plugin::plugin_error::PluginError;
use geo::{coord, Coord};
use routee_compass_core::{
    model::{property::vertex::Vertex, road_network::graph::Graph},
    util::{
        fs::read_utils,
        geo::haversine,
        unit::{Distance, DistanceUnit, BASE_DISTANCE_UNIT},
    },
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
        let vertices = graph.vertices.to_vec();
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
    tolerance: Option<(Distance, DistanceUnit)>,
}

impl RTreePlugin {
    /// creates a new R Tree input plugin instance.
    ///
    /// # Arguments
    ///
    /// * `vertex_file` - file containing vertices
    /// * `tolerance_distance` - optional max distance to nearest vertex (assumed infinity if not included)
    /// * `distance_unit` - distance unit for tolerance, assumed BASE_DISTANCE_UNIT if not provided
    ///
    /// # Returns
    ///
    /// * a plugin instance or an error from file loading
    pub fn new(
        vertex_file: &Path,
        tolerance_distance: Option<Distance>,
        distance_unit: Option<DistanceUnit>,
    ) -> Result<Self, PluginError> {
        let vertices: Box<[Vertex]> =
            read_utils::from_csv(vertex_file, true, None).map_err(PluginError::CsvReadError)?;
        let vertex_rtree = VertexRTree::new(vertices.to_vec());
        let tolerance = match (tolerance_distance, distance_unit) {
            (None, None) => None,
            (None, Some(_)) => None,
            (Some(t), None) => Some((t, BASE_DISTANCE_UNIT)),
            (Some(t), Some(u)) => Some((t, u)),
        };
        Ok(RTreePlugin {
            vertex_rtree,
            tolerance,
        })
    }
}

impl InputPlugin for RTreePlugin {
    /// finds the nearest graph vertex to the user-provided origin (and optionally, destination) coordinates.
    ///
    /// # Arguments
    ///
    /// * `query` - search query assumed to have at least an origin coordinate entry
    ///
    /// # Returns
    ///
    /// * either vertex ids for the nearest coordinates to the the origin (and optionally destination),
    ///   or, an error if not found or not within tolerance
    fn process(&self, query: &serde_json::Value) -> Result<Vec<serde_json::Value>, PluginError> {
        let mut updated = query.clone();
        let src_coord = query.get_origin_coordinate()?;
        let dst_coord_option = query.get_destination_coordinate()?;

        let src_vertex =
            self.vertex_rtree
                .nearest_vertex(src_coord)
                .ok_or(PluginError::PluginFailed(format!(
                    "nearest vertex not found for origin coordinate {:?}",
                    src_coord
                )))?;

        validate_tolerance(src_coord, src_vertex.coordinate, &self.tolerance)?;
        updated.add_origin_vertex(src_vertex.vertex_id)?;

        match dst_coord_option {
            None => {}
            Some(dst_coord) => {
                let dst_vertex = self.vertex_rtree.nearest_vertex(dst_coord).ok_or(
                    PluginError::PluginFailed(format!(
                        "nearest vertex not found for destination coordinate {:?}",
                        dst_coord
                    )),
                )?;
                validate_tolerance(dst_coord, dst_vertex.coordinate, &self.tolerance)?;
                updated.add_destination_vertex(dst_vertex.vertex_id)?;
            }
        }

        Ok(vec![updated])
    }
}

/// confirms that two coordinates are within some stated distance tolerance.
/// if no tolerance is provided, the dst coordinate is assumed to be a valid distance.
///
/// # Arguments
///
/// * `src` - source coordinate
/// * `dst` - destination coordinate that may or may not be within some distance
///           tolerance of the src coordinate
/// * `tolerance` - tolerance parameters set by user for the rtree plugin. if this is None,
///                 all coordinate pairs are assumed to be within distance tolerance, but this
///                 may lead to unexpected behavior where far away coordinates are considered "matched".
///
/// # Returns
///
/// * nothing, or an error if the coordinates are not within tolerance
fn validate_tolerance(
    src: Coord,
    dst: Coord,
    tolerance: &Option<(Distance, DistanceUnit)>,
) -> Result<(), PluginError> {
    match tolerance {
        Some((tolerance_distance, tolerance_distance_unit)) => {
            let distance_meters =
                haversine::coord_distance_meters(src, dst).map_err(PluginError::PluginFailed)?;
            let distance = DistanceUnit::Meters.convert(distance_meters, *tolerance_distance_unit);
            if &distance >= tolerance_distance {
                Err(PluginError::PluginFailed(
                    format!(
                        "coord {:?} nearest vertex coord is {:?} which is {} {} away, exceeding the distance tolerance of {} {}", 
                        src,
                        dst,
                        distance,
                        tolerance_distance_unit,
                        tolerance_distance,
                        tolerance_distance_unit,
                    )
                ))
            } else {
                Ok(())
            }
        }
        None => Ok(()),
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
        let rtree_plugin = RTreePlugin::new(&vertices_filepath, None, None).unwrap();
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
