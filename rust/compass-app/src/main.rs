use clap::Parser;
use compass_app::app::app_error::AppError;
use compass_app::app::compass::compass_app::CompassApp;
use compass_app::app::compass::compass_json_extensions::CompassJsonExtensions;
use compass_app::app::compass::conf_v2::compass_app_builder::CompassAppBuilder;
use compass_app::app::compass::config::compass_app_args::CompassAppArgs;
use config::Config;
use log::info;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    // build CompassApp from config
    let args = CompassAppArgs::parse();
    let defaul_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("app")
        .join("compass")
        .join("config")
        .join("config.default.toml");
    let conf_file = args.config.unwrap_or(defaul_file);
    let config = Config::builder()
        .add_source(config::File::from(conf_file))
        .build()
        .map_err(AppError::ConfigError)?;
    info!("Config: {:?}", config);
    let builder = CompassAppBuilder::default();
    let compass_app = CompassApp::try_from((&config, &builder))?;

    // read user file containing JSON query/queries
    let query_file = File::open(args.query_file)?;
    let reader = BufReader::new(query_file);
    let user_json: serde_json::Value =
        serde_json::from_reader(reader).map_err(AppError::CodecError)?;
    let user_queries = user_json.get_queries()?;
    info!("Query: {:?}", user_json);

    // run searches and return result
    let output_rows = compass_app.run(user_queries)?;
    let output_contents = serde_json::to_string(&output_rows)?;
    std::fs::write("result.json", output_contents)?;

    return Ok(());
}
