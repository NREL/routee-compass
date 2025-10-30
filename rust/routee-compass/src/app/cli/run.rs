use super::cli_args::CliArgs;
use crate::app::compass::CompassAppConfig;
use crate::app::compass::{
    CompassApp, CompassAppError, CompassBuilderInventory, CompassJsonExtensions,
};
use itertools::{Either, Itertools};
use log::{debug, error};
use serde_json::{json, Value};
use std::io::BufRead;
use std::{fs::File, io::BufReader, path::Path};

/// runs CompassApp from the command line using the provided app builder and optional
/// additional CompassApp configuration overwrites.
///
/// # Arguments
/// * `args`       - command line arguments for this run
/// * `builder`    - optional builder instance to overwrite the default. see CompassBuilderInventory for explanation.
/// * `run_config` - optional CompassApp configuration overrides
///
/// # Returns
/// After executing all queries, returns nothing, or returns an un-handled application error.
/// Any user errors are logged and optionally written to an output file depending on the file io policy.
pub fn command_line_runner(
    args: &CliArgs,
    builder: Option<CompassBuilderInventory>,
    run_config: Option<&Value>,
) -> Result<(), CompassAppError> {
    args.validate()?;

    // build the app
    let builder_or_default = match builder {
        Some(b) => b,
        None => CompassBuilderInventory::new()?,
    };
    let config_path = Path::new(&args.config_file);
    let config = CompassAppConfig::try_from(config_path)?;
    let compass_app = match CompassApp::new(&config, &builder_or_default) {
        Ok(app) => app,
        Err(e) => {
            error!("Could not build CompassApp from config file: {e}");
            return Err(e);
        }
    };

    // read user file containing JSON query/queries
    log::info!("reading queries from {}", &args.query_file);
    let query_file = File::open(args.query_file.clone()).map_err(|_e| {
        CompassAppError::BuildFailure(format!("Could not find query file {}", args.query_file))
    })?;

    // execute queries on app
    match (args.chunksize, args.newline_delimited) {
        (None, false) => run_json(&query_file, &compass_app, run_config),
        (Some(_), false) => Err(CompassAppError::InternalError(String::from(
            "not yet implemented",
        ))),
        (_, true) => {
            let chunksize = args.get_chunksize_option()?;
            run_newline_json(&query_file, chunksize, &compass_app, run_config)
        }
    }
}

/// parses a file as a valid JSON object and executes it as queries against
/// the CompassApp.run command.
fn run_json(
    query_file: &File,
    compass_app: &CompassApp,
    run_config: Option<&Value>,
) -> Result<(), CompassAppError> {
    let reader = BufReader::new(query_file);
    let user_json: serde_json::Value = serde_json::from_reader(reader)?;
    let mut user_queries = user_json.get_queries()?;
    let results = compass_app.run(&mut user_queries, run_config)?;
    for result in results.iter() {
        log_error(result);
    }
    Ok(())
}

/// parses a file as newline-delimited JSON which can be optionally chunked into sub-batches
/// and each sub-batch run as queries against the CompassApp.run command.
/// chunksize should be >> the configured CompassApp parallelism (from TOML file) for best
/// performance.
fn run_newline_json(
    query_file: &File,
    chunksize_option: Option<usize>,
    compass_app: &CompassApp,
    run_config: Option<&Value>,
) -> Result<(), CompassAppError> {
    let reader = BufReader::new(query_file);
    let iterator = reader.lines();
    let chunksize = chunksize_option.unwrap_or(usize::MAX);
    let chunks = iterator.chunks(chunksize);
    log::info!("reading {chunksize} queries at-a-time from newline-delimited JSON file");

    for (iteration, chunk) in chunks.into_iter().enumerate() {
        debug!("executing batch {}", iteration + 1);
        // parse JSON output
        let (mut chunk_queries, errors): (Vec<Value>, Vec<CompassAppError>) =
            chunk.enumerate().partition_map(|(idx, row)| match row {
                Ok(string) => match serde_json::from_str(&string) {
                    Ok(query) => Either::Left(query),
                    Err(e) => Either::Right(CompassAppError::CompassFailure(format!(
                        "while reading chunk {iteration} row {idx}, failed to read JSON: {e}"
                    ))),
                },
                Err(e) => Either::Right(CompassAppError::CompassFailure(format!(
                    "failed to parse query row due to: {e}"
                ))),
            });

        // run Compass on this chunk of queries
        for result in compass_app.run(&mut chunk_queries, run_config)?.iter() {
            log_error(result)
        }

        // report JSON parsing errors
        for error in errors {
            let error_json = json!({
                "request": "failed to parse",
                "error": error.to_string()
            });
            log_error(&error_json)
        }
    }

    Ok(())
}

fn log_error(result: &Value) {
    if let Some(error) = result.get("error") {
        let error_string = error.to_string().replace("\\n", "\n");
        error!("Error: {error_string}");
    }
}
