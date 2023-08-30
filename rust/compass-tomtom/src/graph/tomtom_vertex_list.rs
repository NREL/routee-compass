use std::os::unix::process;
use std::{fs::File, io::BufReader};

use compass_core::model::property::vertex::Vertex;
use compass_core::util::fs::read_utils;
use flate2::read::GzDecoder;

use super::{tomtom_graph_config::TomTomGraphConfig, tomtom_graph_error::TomTomGraphError};
use kdam::Bar;
use kdam::BarExt;

pub struct TomTomVertexListConfig<'a> {
    pub config: &'a TomTomGraphConfig,
    pub n_vertices: usize,
}

pub fn read_vertex_list(c: TomTomVertexListConfig) -> Result<Vec<Vertex>, TomTomGraphError> {
    // set up csv.gz reading and row deserialization
    // let vertex_list_file = File::open(c.config.vertex_list_csv.clone())?;

    // let mut vertex_reader =
    //     csv::Reader::from_reader(Box::new(BufReader::new(GzDecoder::new(vertex_list_file))));
    // let vertex_rows = vertex_reader.deserialize();

    let mut processed: usize = 0;
    let mut pb = Bar::builder()
        .total(c.n_vertices)
        .animation("fillup")
        .desc("vertex list")
        .build()
        .map_err(|e| TomTomGraphError::ProgressBarBuildError(String::from("vertex list"), e))?;

    let cb = Box::new(|_v: &Vertex| {
        pb.update(1);
        processed = processed + 1;
    });
    let result: Vec<Vertex> = read_utils::vec_from_csv(
        &c.config.vertex_list_csv,
        true,
        Some(c.n_vertices),
        Some(cb),
    )?;

    print!("\n");
    Ok(result)
}
