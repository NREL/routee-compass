use super::compass_configuration_error::CompassConfigurationError;
use super::compass_configuration_field::CompassConfigurationField;
use serde::de;
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

const FILE_NORMALIZATION_POSTFIX: &str = "_input_file";

pub trait ConfigJsonExtensions {
    fn get_config_section(
        &self,
        section: CompassConfigurationField,
    ) -> Result<serde_json::Value, CompassConfigurationError>;
    fn get_config_path(
        &self,
        key: &dyn AsRef<str>,
        parent_key: &dyn AsRef<str>,
    ) -> Result<PathBuf, CompassConfigurationError>;
    fn get_config_path_optional(
        &self,
        key: &dyn AsRef<str>,
        parent_key: &dyn AsRef<str>,
    ) -> Result<Option<PathBuf>, CompassConfigurationError>;
    fn get_config_string(
        &self,
        key: &dyn AsRef<str>,
        parent_key: &dyn AsRef<str>,
    ) -> Result<String, CompassConfigurationError>;
    fn get_config_array(
        &self,
        key: &dyn AsRef<str>,
        parent_key: &dyn AsRef<str>,
    ) -> Result<Vec<serde_json::Value>, CompassConfigurationError>;
    fn get_config_i64(
        &self,
        key: &dyn AsRef<str>,
        parent_key: &dyn AsRef<str>,
    ) -> Result<i64, CompassConfigurationError>;
    fn get_config_f64(
        &self,
        key: &dyn AsRef<str>,
        parent_key: &dyn AsRef<str>,
    ) -> Result<f64, CompassConfigurationError>;
    fn get_config_from_str<T: FromStr>(
        &self,
        key: &dyn AsRef<str>,
        parent_key: &dyn AsRef<str>,
    ) -> Result<T, CompassConfigurationError>;
    fn get_config_serde<T: de::DeserializeOwned>(
        &self,
        key: &dyn AsRef<str>,
        parent_key: &dyn AsRef<str>,
    ) -> Result<T, CompassConfigurationError>;
    fn get_config_serde_optional<T: de::DeserializeOwned>(
        &self,
        key: &dyn AsRef<str>,
        parent_key: &dyn AsRef<str>,
    ) -> Result<Option<T>, CompassConfigurationError>;
    fn normalize_file_paths(
        &self,
        parent_key: &dyn AsRef<str>,
        root_config_path: &Path,
    ) -> Result<serde_json::Value, CompassConfigurationError>;
}

impl ConfigJsonExtensions for serde_json::Value {
    fn get_config_section(
        &self,
        section: CompassConfigurationField,
    ) -> Result<serde_json::Value, CompassConfigurationError> {
        let section = self
            .get(section.to_str())
            .ok_or_else(|| {
                CompassConfigurationError::ExpectedFieldForComponent(
                    section.to_string(),
                    String::from(""),
                )
            })?
            .clone();

        Ok(section)
    }
    fn get_config_path_optional(
        &self,
        key: &dyn AsRef<str>,
        parent_key: &dyn AsRef<str>,
    ) -> Result<Option<PathBuf>, CompassConfigurationError> {
        match self.get(key.as_ref()) {
            None => Ok(None),
            Some(_) => {
                let config_path = self.get_config_path(key, parent_key)?;
                Ok(Some(config_path))
            }
        }
    }
    fn get_config_path(
        &self,
        key: &dyn AsRef<str>,
        parent_key: &dyn AsRef<str>,
    ) -> Result<PathBuf, CompassConfigurationError> {
        let path_string = self.get_config_string(key, parent_key)?;
        let path = PathBuf::from(&path_string);

        // if file can be found, just it
        if path.is_file() {
            Ok(path)
        } else {
            // can't find the file
            Err(CompassConfigurationError::FileNotFoundForComponent(
                path_string,
                String::from(key.as_ref()),
                String::from(parent_key.as_ref()),
            ))
        }
    }
    fn get_config_string(
        &self,
        key: &dyn AsRef<str>,
        parent_key: &dyn AsRef<str>,
    ) -> Result<String, CompassConfigurationError> {
        let value = self
            .get(key.as_ref())
            .ok_or_else(|| {
                CompassConfigurationError::ExpectedFieldForComponent(
                    String::from(key.as_ref()),
                    String::from(parent_key.as_ref()),
                )
            })?
            .as_str()
            .map(String::from)
            .ok_or_else(|| {
                CompassConfigurationError::ExpectedFieldWithType(
                    String::from(key.as_ref()),
                    String::from("String"),
                )
            })?;
        Ok(value)
    }

    fn get_config_array(
        &self,
        key: &dyn AsRef<str>,
        parent_key: &dyn AsRef<str>,
    ) -> Result<Vec<serde_json::Value>, CompassConfigurationError> {
        let array = self
            .get(key.as_ref())
            .ok_or_else(|| {
                CompassConfigurationError::ExpectedFieldForComponent(
                    String::from(key.as_ref()),
                    String::from(parent_key.as_ref()),
                )
            })?
            .as_array()
            .ok_or_else(|| {
                CompassConfigurationError::ExpectedFieldWithType(
                    String::from(key.as_ref()),
                    String::from("Array"),
                )
            })?
            .to_owned();
        Ok(array)
    }

    fn get_config_i64(
        &self,
        key: &dyn AsRef<str>,
        parent_key: &dyn AsRef<str>,
    ) -> Result<i64, CompassConfigurationError> {
        let value = self
            .get(key.as_ref())
            .ok_or_else(|| {
                CompassConfigurationError::ExpectedFieldForComponent(
                    String::from(key.as_ref()),
                    String::from(parent_key.as_ref()),
                )
            })?
            .as_i64()
            .ok_or_else(|| {
                CompassConfigurationError::ExpectedFieldWithType(
                    String::from(key.as_ref()),
                    String::from("64-bit signed integer"),
                )
            })?;
        Ok(value)
    }

    fn get_config_f64(
        &self,
        key: &dyn AsRef<str>,
        parent_key: &dyn AsRef<str>,
    ) -> Result<f64, CompassConfigurationError> {
        let value = self
            .get(key.as_ref())
            .ok_or_else(|| {
                CompassConfigurationError::ExpectedFieldForComponent(
                    String::from(key.as_ref()),
                    String::from(parent_key.as_ref()),
                )
            })?
            .as_f64()
            .ok_or_else(|| {
                CompassConfigurationError::ExpectedFieldWithType(
                    String::from(key.as_ref()),
                    String::from("64-bit floating point"),
                )
            })?;
        Ok(value)
    }

    fn get_config_from_str<T: FromStr>(
        &self,
        key: &dyn AsRef<str>,
        parent_key: &dyn AsRef<str>,
    ) -> Result<T, CompassConfigurationError> {
        let value = self
            .get(key.as_ref())
            .ok_or_else(|| {
                CompassConfigurationError::ExpectedFieldForComponent(
                    String::from(key.as_ref()),
                    String::from(parent_key.as_ref()),
                )
            })?
            .as_str()
            .ok_or_else(|| {
                CompassConfigurationError::ExpectedFieldWithType(
                    String::from(key.as_ref()),
                    String::from("string-parseable"),
                )
            })?;
        let result = T::from_str(value).map_err(|_| {
            CompassConfigurationError::ExpectedFieldWithType(
                String::from(key.as_ref()),
                format!("failed to parse type from string {}", value),
            )
        })?;
        Ok(result)
    }

    fn get_config_serde<T: de::DeserializeOwned>(
        &self,
        key: &dyn AsRef<str>,
        parent_key: &dyn AsRef<str>,
    ) -> Result<T, CompassConfigurationError> {
        let value = self
            .get(key.as_ref())
            .ok_or_else(|| {
                CompassConfigurationError::ExpectedFieldForComponent(
                    String::from(key.as_ref()),
                    String::from(parent_key.as_ref()),
                )
            })?
            .to_owned();

        let result: T = serde_json::from_value(value)
            .map_err(CompassConfigurationError::SerdeDeserializationError)?;
        Ok(result)
    }
    fn get_config_serde_optional<T: de::DeserializeOwned>(
        &self,
        key: &dyn AsRef<str>,
        _parent_key: &dyn AsRef<str>,
    ) -> Result<Option<T>, CompassConfigurationError> {
        match self.get(key.as_ref()) {
            None => Ok(None),
            Some(value) => {
                let result: T = serde_json::from_value(value.to_owned())
                    .map_err(CompassConfigurationError::SerdeDeserializationError)?;
                Ok(Some(result))
            }
        }
    }

    /// This function is used to normalize file paths in the configuration JSON object.
    /// Incoming file paths can be in one of three locations:
    ///
    /// 1. Absolute path
    /// 2. Relative path to the config file itself
    /// 3. Relative path to where the application is being run
    ///
    /// This function scans each key value pair in the config and for any key that
    /// ends with `_input_file`, it will attempt to normalize the path such that the application
    /// can find the file regardless of where it is being executed.
    ///
    /// Arguments:
    ///
    /// * `root_config_path` - The path to the where the config file is located.
    ///
    /// Returns:
    ///
    /// * `Result<serde_json::Value, CompassConfigurationError>` - The JSON object with normalized paths.
    fn normalize_file_paths(
        &self,
        parent_key: &dyn AsRef<str>,
        root_config_path: &Path,
    ) -> Result<serde_json::Value, CompassConfigurationError> {
        match self {
            serde_json::Value::String(path_string) => {
                let path = Path::new(path_string);

                // no need to modify if the file exists
                if path.is_file() {
                    Ok(serde_json::Value::String(path_string.clone()))
                } else {
                    // next we try adding the root config path and see if that exists
                    let root_config_parent = match root_config_path.parent() {
                        Some(parent) => parent,
                        None => Path::new(""),
                    };
                    let new_path = root_config_parent.join(path);
                    let new_path_string = new_path
                        .to_str()
                        .ok_or_else(|| {
                            CompassConfigurationError::FileNormalizationError(path_string.clone())
                        })?
                        .to_string();
                    if new_path.is_file() {
                        Ok(serde_json::Value::String(new_path_string))
                    } else {
                        // if we can't find the file in either location, we throw an error
                        Err(CompassConfigurationError::FileNormalizationNotFound(
                            String::from(parent_key.as_ref()),
                            path_string.clone(),
                            new_path_string,
                        ))
                    }
                }
            }
            serde_json::Value::Object(obj) => {
                let mut new_obj = serde_json::map::Map::new();
                for (key, value) in obj.iter() {
                    if key.ends_with(FILE_NORMALIZATION_POSTFIX)
                        || value.is_object()
                        || value.is_array()
                    {
                        new_obj.insert(
                            String::from(key),
                            value.normalize_file_paths(key, root_config_path)?,
                        );
                    } else {
                        new_obj.insert(String::from(key), value.clone());
                    }
                }
                Ok(serde_json::Value::Object(new_obj))
            }
            serde_json::Value::Array(arr) => {
                let mut new_arr = Vec::new();
                for value in arr.iter() {
                    match value {
                        serde_json::Value::Array(_) => {
                            new_arr.push(value.normalize_file_paths(parent_key, root_config_path)?)
                        }
                        serde_json::Value::Object(_) => {
                            new_arr.push(value.normalize_file_paths(parent_key, root_config_path)?)
                        }
                        _ => new_arr.push(value.clone()),
                    }
                }
                Ok(serde_json::Value::Array(new_arr))
            }
            _ => Ok(self.clone()),
        }
    }
}
