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
        .ok_or_else(|| {
            CompassAppError::InternalError("Could not parse incoming config file path".to_string())
        })?
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
///
/// # Arguments
///
/// * `queries` - user queries to load balance based on a query weight heuristic.
/// * `parallelism` - number of chunks to split inputs into, set by user
/// * `default` - weight value if weight heuristic fails to produce an estimate
///
/// # Returns
///
/// An index for sorting the values so that, when fed into rayon's par_chunks iterator,
/// load balances the queries across processes based on the estimates. the resulting
/// batches are not equal-sized
pub fn apply_load_balancing_policy(
    queries: &[serde_json::Value],
    parallelism: usize,
    default: f64,
) -> Result<Vec<Vec<&serde_json::Value>>, CompassAppError> {
    if queries.is_empty() {
        return Ok(vec![]);
    }
    let mut bin_totals = vec![0.0; parallelism];
    let mut assignments: Vec<Vec<&serde_json::Value>> = vec![vec![]; parallelism];
    for q in queries.iter() {
        let w = q.get_query_weight_estimate()?.unwrap_or(default);
        let min_bin = min_bin(&bin_totals)?;
        bin_totals[min_bin] += w;
        assignments[min_bin].push(q);
    }
    Ok(assignments)
}

fn min_bin(bins: &[f64]) -> Result<usize, PluginError> {
    bins.iter()
        .enumerate()
        .min_by_key(|(_i, w)| OrderedFloat(**w))
        .map(|(i, _w)| i)
        .ok_or_else(|| {
            PluginError::InternalError(String::from("cannot find min bin of empty slice"))
        })
}

#[cfg(test)]
mod test {
    use super::apply_load_balancing_policy;
    use crate::plugin::input::input_field::InputField;
    use serde_json::json;

    fn test_run_policy(queries: Vec<serde_json::Value>, parallelism: usize) -> Vec<Vec<i64>> {
        apply_load_balancing_policy(&queries, parallelism, 1.0)
            .unwrap()
            .iter()
            .map(|qs| {
                let is: Vec<i64> = qs
                    .iter()
                    .map(|q| q.get("index").unwrap().as_i64().unwrap())
                    .collect();
                is
            })
            .collect::<Vec<_>>()
    }

    #[test]
    fn test_uniform_input() {
        // striped
        let queries: Vec<serde_json::Value> = (0..12)
            .map(|i| {
                json!({
                    "index": i,
                    InputField::QueryWeightEstimate.to_str(): 1
                })
            })
            .collect();
        let parallelism = 4;
        let result = test_run_policy(queries, parallelism);
        let expected: Vec<Vec<i64>> =
            vec![vec![0, 4, 8], vec![1, 5, 9], vec![2, 6, 10], vec![3, 7, 11]];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_incremental_input() {
        // this produces the same layout as the uniform input
        let queries: Vec<serde_json::Value> = (0..12)
            .map(|i| {
                json!({
                    "index": i,
                    InputField::QueryWeightEstimate.to_str(): i + 1
                })
            })
            .collect();
        let parallelism = 4;
        let result = test_run_policy(queries, parallelism);
        let expected: Vec<Vec<i64>> =
            vec![vec![0, 4, 8], vec![1, 5, 9], vec![2, 6, 10], vec![3, 7, 11]];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_cycling_input() {
        // an input one can verify via debugging that produces the expected output below
        let queries: Vec<serde_json::Value> = [1, 4, 1, 2, 1, 4, 1, 2, 1, 4, 1, 2]
            .iter()
            .enumerate()
            .map(|(i, estimate)| {
                json!({
                    "index": i,
                    InputField::QueryWeightEstimate.to_str(): estimate
                })
            })
            .collect();
        let parallelism = 4;
        let result = test_run_policy(queries, parallelism);
        let expected = vec![vec![0, 4, 6, 8, 9], vec![1, 10], vec![2, 5], vec![3, 7, 11]];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_big_outlier() {
        let queries: Vec<serde_json::Value> = [4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]
            .iter()
            .enumerate()
            .map(|(idx, est)| {
                json!({
                    "index": idx,
                    InputField::QueryWeightEstimate.to_str(): est
                })
            })
            .collect();
        let parallelism = 4;
        let result = test_run_policy(queries, parallelism);
        let expected = vec![vec![0], vec![1, 4, 7, 10], vec![2, 5, 8, 11], vec![3, 6, 9]];
        assert_eq!(result, expected);
    }
}
