use std::path::PathBuf;

use crate::model::property::vertex::Vertex;
use crate::model::road_network::graph_error::GraphError;
use crate::util::fs::read_utils;
use kdam::{Bar, BarExt};

pub struct VertexLoaderConfig {
    pub vertex_list_csv: PathBuf,
    pub n_vertices: usize,
}

impl TryFrom<VertexLoaderConfig> for Box<[Vertex]> {
    type Error = GraphError;

    fn try_from(conf: VertexLoaderConfig) -> Result<Self, Self::Error> {
        let mut processed: usize = 0;
        let mut pb = Bar::builder()
            .total(conf.n_vertices)
            .animation("fillup")
            .desc("vertex list")
            .build()
            .map_err(|e| GraphError::ProgressBarBuildError(String::from("vertex list"), e))?;

        let cb = Box::new(|_v: &Vertex| {
            let _ = pb.update(1);
            processed += 1;
        });
        let result: Box<[Vertex]> = read_utils::from_csv(&conf.vertex_list_csv, true, Some(cb))?;

        println!();
        Ok(result)
    }
}
