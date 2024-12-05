use std::sync::Arc;

use super::map_error::MapError;
use crate::{
    model::network::{EdgeId, Graph},
    util::{fs::read_utils, geo::geo_io_utils},
};
use geo::LineString;
use kdam::BarExt;

pub struct GeometryModel(Vec<LineString<f32>>);

impl GeometryModel {
    /// with no provided geometries, create minimal LineStrings from pairs of vertex Points
    pub fn new_from_vertices(graph: Arc<Graph>) -> Result<GeometryModel, MapError> {
        let edges = create_linestrings_from_vertices(graph)?;
        Ok(GeometryModel(edges))
    }

    /// use a user-provided enumerated textfile input to load LineString geometries
    pub fn new_from_edges(
        geometry_input_file: &String,
        graph: Arc<Graph>,
    ) -> Result<GeometryModel, MapError> {
        let edges = read_linestrings(geometry_input_file, graph.edges.len())?;
        Ok(GeometryModel(edges))
    }

    /// iterate through the geometries of this model
    pub fn geometries<'a>(&'a self) -> Box<dyn Iterator<Item = &'a LineString<f32>> + 'a> {
        Box::new(self.0.iter())
    }

    /// get a single geometry by it's EdgeId
    pub fn get<'a>(&'a self, edge_id: &EdgeId) -> Result<&'a LineString<f32>, MapError> {
        self.0
            .get(edge_id.0)
            .ok_or(MapError::MissingEdgeId(*edge_id))
    }
}

fn read_linestrings(
    geometry_input_file: &String,
    n_edges: usize,
) -> Result<Vec<geo::LineString<f32>>, MapError> {
    let mut pb = kdam::Bar::builder()
        .total(n_edges)
        .animation("fillup")
        .desc("edge LineString geometry file")
        .build()
        .map_err(MapError::InternalError)?;

    let cb = Box::new(|| {
        let _ = pb.update(1);
    });
    let geoms = read_utils::read_raw_file(
        geometry_input_file,
        geo_io_utils::parse_wkt_linestring,
        Some(cb),
    )
    .map_err(|e: std::io::Error| {
        MapError::BuildError(format!("error loading {}: {}", geometry_input_file, e))
    })?
    .to_vec();
    eprintln!();
    Ok(geoms)
}

fn create_linestrings_from_vertices(graph: Arc<Graph>) -> Result<Vec<LineString<f32>>, MapError> {
    let n_edges = graph.edges.len();
    let mut pb = kdam::Bar::builder()
        .total(n_edges)
        .animation("fillup")
        .desc("edge LineString geometry file")
        .build()
        .map_err(MapError::InternalError)?;

    let edges = graph
        .edges
        .iter()
        .map(|e| {
            let src_v = graph.get_vertex(&e.src_vertex_id).map_err(|_| {
                MapError::InternalError(format!(
                    "edge {} src vertex {} missing",
                    e.edge_id, e.src_vertex_id
                ))
            })?;
            let dst_v = graph.get_vertex(&e.dst_vertex_id).map_err(|_| {
                MapError::InternalError(format!(
                    "edge {} dst vertex {} missing",
                    e.edge_id, e.dst_vertex_id
                ))
            })?;

            let linestring = geo::line_string![src_v.coordinate.0, dst_v.coordinate.0,];
            let _ = pb.update(1);
            Ok(linestring)
        })
        .collect::<Result<Vec<_>, MapError>>()?;

    eprintln!();
    Ok(edges)
}
