use std::{fs::File, io::BufReader};

use compass_core::model::property::vertex::Vertex;
use flate2::read::GzDecoder;

use super::{tomtom_graph_config::TomTomGraphConfig, tomtom_graph_error::TomTomGraphError};
use kdam::Bar;
use kdam::BarExt;

pub struct TomTomVertexListConfig<'a> {
    pub config: &'a TomTomGraphConfig,
    pub n_vertices: usize,
}

pub fn read_vertex_list(c: TomTomVertexListConfig) -> Result<Vec<Vertex>, TomTomGraphError> {
    // build collections to store in the TomTomGraph
    let mut vertices: Vec<Vertex> = vec![Vertex::default(); c.n_vertices];

    // set up csv.gz reading and row deserialization
    let vertex_list_file = File::open(c.config.vertex_list_csv.clone())
        .map_err(|e| TomTomGraphError::IOError { source: e })?;
    let mut vertex_reader =
        csv::Reader::from_reader(Box::new(BufReader::new(GzDecoder::new(vertex_list_file))));
    let vertex_rows = vertex_reader.deserialize();

    let mut pb = Bar::builder()
        .total(c.n_vertices)
        .animation("fillup")
        .desc("vertex list")
        .build()
        .map_err(|e| TomTomGraphError::ProgressBarBuildError(String::from("vertex list"), e))?;

    for row in vertex_rows {
        let vertex: Vertex = row.map_err(|e| TomTomGraphError::CsvError { source: e })?;
        vertices[vertex.vertex_id.0 as usize] = vertex;
        pb.update(1);
    }
    print!("\n");
    Ok(vertices)
}
