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

// TODO:
//   - the API for OutputPlugin now expects a SearchInstance which is non-trivial to instantiate.
//   the logic for adding geometries should be refactored into a separate function and this test
//   should be moved to the file where that function exists.
//   - the loading of geometries is now handled by the MapModel. testing geometry retrieval and
//   linestring reconstruction should be moved to the map_model.rs file.

#[cfg(test)]
mod tests {

    use routee_compass_core::util::{
        fs::read_utils::read_raw_file, geo::geo_io_utils::parse_wkt_linestring,
    };

    use std::path::PathBuf;

    fn mock_geometry_file() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("model")
            .join("map")
            .join("test")
            .join("geometries.txt")
    }

    fn mock_graph() -> Graph {
        todo!()
    }

    #[test]
    fn test_geometry_deserialization() {
        let result = read_raw_file(mock_geometry_file(), parse_wkt_linestring, None).unwrap();
        assert_eq!(result.len(), 3);
    }

    // #[test]
    // fn test_add_geometry() {
    //     let expected_geometry = String::from("LINESTRING(0 0,1 1,2 2,3 3,4 4,5 5,6 6,7 7,8 8)");
    //     let mut output_result = serde_json::json!({});
    //     let route = vec![
    //         EdgeTraversal {
    //             edge_id: EdgeId(0),
    //             access_cost: Cost::from(0.0),
    //             traversal_cost: Cost::from(0.0),
    //             result_state: vec![StateVar(0.0)],
    //         },
    //         EdgeTraversal {
    //             edge_id: EdgeId(1),
    //             access_cost: Cost::from(0.0),
    //             traversal_cost: Cost::from(0.0),
    //             result_state: vec![StateVar(0.0)],
    //         },
    //         EdgeTraversal {
    //             edge_id: EdgeId(2),
    //             access_cost: Cost::from(0.0),
    //             traversal_cost: Cost::from(0.0),
    //             result_state: vec![StateVar(0.0)],
    //         },
    //     ];
    //     let search_result = SearchAppResult {
    //         route,
    //         tree: HashMap::new(),
    //         search_executed_time: Local::now().to_rfc3339(),
    //         algorithm_runtime: Duration::ZERO,
    //         route_runtime: Duration::ZERO,
    //         search_app_runtime: Duration::ZERO,
    //         iterations: 0,
    //     };
    //     let filename = mock_geometry_file();
    //     let _route_geometry = true;
    //     let _tree_geometry = false;
    //     let geom_plugin =
    //         TraversalPlugin::from_file(&filename, Some(TraversalOutputFormat::Wkt), None).unwrap();

    //     geom_plugin
    //         .process(&mut output_result, &Ok(search_result))
    //         .unwrap();
    //     let geometry_wkt = output_result.get_route_geometry_wkt().unwrap();
    //     assert_eq!(geometry_wkt, expected_geometry);
    // }
}
