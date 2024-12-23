use std::path::PathBuf;

use super::{NetworkError, Vertex};
use crate::util::fs::read_utils;
use kdam::{Bar, BarExt};

pub struct VertexLoaderConfig {
    pub vertex_list_csv: PathBuf,
    pub n_vertices: usize,
}

impl TryFrom<VertexLoaderConfig> for Box<[Vertex]> {
    type Error = NetworkError;

    fn try_from(conf: VertexLoaderConfig) -> Result<Self, Self::Error> {
        let mut processed: usize = 0;
        let mut pb = Bar::builder()
            .total(conf.n_vertices)
            .animation("fillup")
            .desc("vertex list")
            .build()
            .map_err(|e| {
                NetworkError::InternalError(format!("could not build progress bar: {}", e))
            })?;

        let cb = Box::new(|_v: &Vertex| {
            let _ = pb.update(1);
            processed += 1;
        });
        let result: Box<[Vertex]> = read_utils::from_csv(&conf.vertex_list_csv, true, Some(cb))?;

        eprintln!();
        Ok(result)
    }
}
