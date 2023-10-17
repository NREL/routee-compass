use clap::Parser;

use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CompassAppArgs {
    pub query_file: PathBuf,
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,
}
