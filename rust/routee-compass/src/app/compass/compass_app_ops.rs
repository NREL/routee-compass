use super::{compass_app_error::CompassAppError, compass_input_field::CompassInputField};
use crate::plugin::{input::input_json_extensions::InputJsonExtensions, plugin_error::PluginError};
use config::Config;
use ordered_float::OrderedFloat;
use std::path::Path;

/// reads the compass configuration TOML file from a path
/// combines it with a configuration file that provides library defaults
///
/// # Arguments
///
/// * `config` - path to the config file
///
/// # Returns
///
/// A config object read from file, or an error
pub fn read_config_from_file(config_path: &Path) -> Result<Config, CompassAppError> {
    let default_config = config::File::from_str(
        include_str!("config.default.toml"),
        config::FileFormat::Toml,
    );

    // We want to store the location of where the config file
    // was found so we can use it later to resolve relative paths
    let conf_file_string = config_path
        .to_str()
        .ok_or(CompassAppError::InternalError(
            "Could not parse incoming config file path".to_string(),
        ))?
        .to_string();

    let config = Config::builder()
        .add_source(default_config)
        .add_source(config::File::from(config_path))
        .set_override(
            CompassInputField::ConfigInputFile.to_string(),
            conf_file_string,
        )?
        .build()
        .map_err(CompassAppError::ConfigError)?;

    Ok(config)
}

/// Reads a configuration file from a deserializable string in the specified format.
/// This also requires the file path of where the string was loaded from since we use that
/// to normalize paths later.
///
/// # Arguments
///
/// * `config_as_string` - the configuration file as a string
/// * `format` - the format of the string
/// * `original_file_path` - the path to the file that was loaded
///
/// # Returns
///
/// A config object read from file, or an error
pub fn read_config_from_string(
    config_as_string: String,
    format: config::FileFormat,
    original_file_path: String,
) -> Result<Config, CompassAppError> {
    let default_config = config::File::from_str(
        include_str!("config.default.toml"),
        config::FileFormat::Toml,
    );

    let user_config = config::File::from_str(&config_as_string, format);

    let config = Config::builder()
        .add_source(default_config)
        .add_source(user_config)
        .set_override(
            CompassInputField::ConfigInputFile.to_string(),
            original_file_path,
        )?
        .build()
        .map_err(CompassAppError::ConfigError)?;

    Ok(config)
}

/// applies the weight balancing policy set by the LoadBalancerPlugin InputPlugin.
/// sorts all queries in ascending order then lays them (striped by weight) in bins
/// so that each incremental cost weight is fairly assigned.
///
/// todo: an ideal version checks every $paralellism steps during assignment, sums
/// values in each bin, and re-sorts the bin ordering to ensure the fairest assignment.
pub fn construct_load_balancing_index(
    queries: &Vec<serde_json::Value>,
    parallelism: usize,
) -> Result<Vec<usize>, CompassAppError> {
    let mut weighted = queries
        .iter()
        .enumerate()
        .map(|(idx, q)| {
            let w = q.get_query_weight_estimate()?.unwrap_or(1.0);
            Ok((w, idx))
        })
        .collect::<Result<Vec<(f64, usize)>, PluginError>>()
        .map_err(CompassAppError::PluginError)?;
    weighted.sort_by_key(|(w, _idx)| OrderedFloat(*w));
    let mut bins: Vec<Vec<usize>> = vec![vec![]; parallelism];
    for (_w, idx) in weighted.iter() {
        let bin = idx % parallelism;
        bins[bin].push(*idx);
    }
    let result = bins
        .into_iter()
        .flatten()
        // .flat_map(|idx| queries.)
        .collect();
    Ok(result)
}
