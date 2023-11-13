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
pub fn read_config(config: &Path) -> Result<Config, CompassAppError> {
    let default_config = config::File::from_str(
        include_str!("config.default.toml"),
        config::FileFormat::Toml,
    );

    // We want to store the location of where the config file
    // was found so we can use it later to resolve relative paths
    let conf_file_string = config
        .to_str()
        .ok_or(CompassAppError::InternalError(
            "Could not parse incoming config file path".to_string(),
        ))?
        .to_string();

    let config = Config::builder()
        .add_source(default_config)
        .add_source(config::File::from(config))
        .set_override(
            CompassInputField::ConfigInputFile.to_string(),
            conf_file_string,
        )?
        .build()
        .map_err(CompassAppError::ConfigError)?;

    Ok(config)
}
