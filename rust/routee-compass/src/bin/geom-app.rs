use clap::Parser;
use routee_compass::app::geom::geom_app::{GeomApp, GeomAppConfig};
use std::error::Error;
use wkt::ToWkt;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct GeomAppCliArgs {
    pub geom_file: String,
    pub source_dir: String,
    pub n_files: u64,
}

/// simple application to look up Edge WKTs. expects a batch of n_files
/// each flat files with names "edge_ids_{i}.txt", where i = [0, n) and where
/// each row is a number corresponding to an EdgeId.
/// outputs a flat file of LINESTRING WKTs (not quoted for csv).
fn main() -> Result<(), Box<dyn Error>> {
    let args = GeomAppCliArgs::parse();
    let geom_app_conf = GeomAppConfig {
        edge_file: args.geom_file,
    };
    let geom_app = GeomApp::try_from(&geom_app_conf)?;
    for idx in 0..args.n_files {
        let tree_file = format!("{}/edge_ids_{}.txt", args.source_dir, idx);
        let result_file = format!("{}/edges_wkt_{}.txt", args.source_dir, idx);
        let result = geom_app.run(tree_file)?;
        let output = result
            .iter()
            .map(|g| g.wkt_string())
            .collect::<Vec<String>>()
            .join("\n");
        std::fs::write(result_file, output)?;
    }
    Ok(())
}
