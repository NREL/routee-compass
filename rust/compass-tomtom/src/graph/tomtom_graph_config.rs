use compass_core::util::fs_utils::line_count;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct TomTomGraphConfig {
    pub edge_list_csv: String,
    pub vertex_list_csv: String,
    pub n_edges: Option<usize>,
    pub n_vertices: Option<usize>,
}

impl TomTomGraphConfig {
    pub fn get_n_edges(&self) -> std::io::Result<usize> {
        match self.n_edges {
            Some(n) => Ok(n),
            None => {
                let is_gzip = self.edge_list_csv.ends_with(".gz");
                let n = line_count(self.edge_list_csv.clone(), is_gzip)?;
                Ok(n)
            }
        }
    }

    pub fn get_n_vertices(&self) -> std::io::Result<usize> {
        match self.n_vertices {
            Some(n) => Ok(n),
            None => {
                let is_gzip = self.vertex_list_csv.ends_with(".gz");
                let n = line_count(self.vertex_list_csv.clone(), is_gzip)?;
                Ok(n)
            }
        }
    }
}
