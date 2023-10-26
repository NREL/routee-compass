use clap::Parser;
use log::info;
use routee_compass::app::compass::compass_app::CompassApp;
use routee_compass::app::compass::compass_app_args::CompassAppArgs;
use routee_compass::app::compass::compass_json_extensions::CompassJsonExtensions;
use routee_compass::app::compass_app_error::CompassAppError;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    // build CompassApp from config
    let args = CompassAppArgs::parse();

    let conf_file = args.config.ok_or(CompassAppError::NoInputFile(
        "No configuration file specified".to_string(),
    ))?;

    let compass_app = CompassApp::try_from(conf_file)?;

    // read user file containing JSON query/queries
    let query_file = File::open(args.query_file.clone()).map_err(|_e| {
        CompassAppError::NoInputFile(format!(
            "Could not find query file {}",
            args.query_file.display()
        ))
    })?;
    let reader = BufReader::new(query_file);
    let user_json: serde_json::Value =
        serde_json::from_reader(reader).map_err(CompassAppError::CodecError)?;
    let user_queries = user_json.get_queries()?;
    info!("Query: {:?}", user_json);

    // run searches and return result
    let output_rows = compass_app.run(user_queries)?;
    let output_contents = serde_json::to_string(&output_rows)?;
    std::fs::write("result.json", output_contents)?;

    return Ok(());
}
