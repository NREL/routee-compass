use super::{compass_app_error::CompassAppError, compass_input_field::CompassInputField};
use config::Config;
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
