use crate::model::graph::graph_config::GraphConfig;
use crate::model::graph::graph_error::GraphError;
use crate::model::property::vertex::Vertex;
use crate::util::fs::read_utils;
use kdam::{Bar, BarExt};

pub struct VertexLoaderConfig<'a> {
    pub config: &'a GraphConfig,
    pub n_vertices: usize,
}

impl<'a> TryFrom<VertexLoaderConfig<'a>> for Vec<Vertex> {
    type Error = GraphError;

    fn try_from(conf: VertexLoaderConfig<'a>) -> Result<Self, Self::Error> {
        let mut processed: usize = 0;
        let mut pb = Bar::builder()
            .total(conf.n_vertices)
            .animation("fillup")
            .desc("vertex list")
            .build()
            .map_err(|e| GraphError::ProgressBarBuildError(String::from("vertex list"), e))?;

        let cb = Box::new(|_v: &Vertex| {
            pb.update(1);
            processed = processed + 1;
        });
        let result: Vec<Vertex> = read_utils::vec_from_csv(
            &conf.config.vertex_list_csv,
            true,
            Some(conf.n_vertices),
            Some(cb),
        )?;

        print!("\n");
        Ok(result)
    }
}
