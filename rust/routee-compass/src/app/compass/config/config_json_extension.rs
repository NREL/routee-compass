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
    fn get_config_path<A: Into<String>, B: Into<String>>(
        &self,
        key: &A,
        parent_key: &B,
    ) -> Result<PathBuf, CompassConfigurationError>;
    fn get_config_path_optional<A: Into<String>, B: Into<String>>(
        &self,
        key: &A,
        parent_key: &B,
    ) -> Result<Option<PathBuf>, CompassConfigurationError>;
    fn get_config_string<A: Into<String>, B: Into<String>>(
        &self,
        key: &A,
        parent_key: &B,
    ) -> Result<String, CompassConfigurationError>;
    fn get_config_array<A: Into<String>, B: Into<String>>(
        &self,
        key: &A,
        parent_key: &B,
    ) -> Result<Vec<serde_json::Value>, CompassConfigurationError>;
    fn get_config_i64<A: Into<String>, B: Into<String>>(
        &self,
        key: &A,
        parent_key: &B,
    ) -> Result<i64, CompassConfigurationError>;
    fn get_config_f64<A: Into<String>, B: Into<String>>(
        &self,
        key: &A,
        parent_key: &B,
    ) -> Result<f64, CompassConfigurationError>;
    fn get_config_from_str<A: Into<String>, B: Into<String>, T: FromStr>(
        &self,
        key: &A,
        parent_key: &B,
    ) -> Result<T, CompassConfigurationError>;
    fn get_config_serde<A: Into<String>, B: Into<String>, T: de::DeserializeOwned>(
        &self,
        key: &A,
        parent_key: &B,
    ) -> Result<T, CompassConfigurationError>;
    fn get_config_serde_optional<A: Into<String>, B: Into<String>, T: de::DeserializeOwned>(
        &self,
        key: &A,
        parent_key: &B,
    ) -> Result<Option<T>, CompassConfigurationError>;
    fn normalize_file_paths<S: Into<String>>(
        &self,
        parent_key: &S,
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
            .ok_or(CompassConfigurationError::ExpectedFieldForComponent(
                section.to_string(),
                String::from(""),
            ))?
            .clone();

        Ok(section)
    }
    fn get_config_path_optional<A: Into<String>, B: Into<String>>(
        &self,
        key: &A,
        parent_key: &B,
    ) -> Result<Option<PathBuf>, CompassConfigurationError> {
        let index: String = (*key).into();
        match self.get(index) {
            None => Ok(None),
            Some(_) => {
                let config_path = self.get_config_path(key, parent_key)?;
                Ok(Some(config_path))
            }
        }
    }
    fn get_config_path<A: Into<String>, B: Into<String>>(
        &self,
        key: &A,
        parent_key: &B,
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
                (*key).into(),
                (*parent_key).into(),
            ))
        }
    }
    fn get_config_string<A: Into<String>, B: Into<String>>(
        &self,
        key: &A,
        parent_key: &B,
    ) -> Result<String, CompassConfigurationError> {
        let index: String = (*key).into();
        let value = self
            .get(index)
            .ok_or(CompassConfigurationError::ExpectedFieldForComponent(
                (*key).into(),
                (*parent_key).into(),
            ))?
            .as_str()
            .map(String::from)
            .ok_or(CompassConfigurationError::ExpectedFieldWithType(
                (*key).into(),
                String::from("String"),
            ))?;
        Ok(value)
    }

    fn get_config_array<A: Into<String>, B: Into<String>>(
        &self,
        key: &A,
        parent_key: &B,
    ) -> Result<Vec<serde_json::Value>, CompassConfigurationError> {
        let index: String = (*key).into();
        let array = self
            .get(index)
            .ok_or(CompassConfigurationError::ExpectedFieldForComponent(
                (*key).into(),
                (*parent_key).into(),
            ))?
            .as_array()
            .ok_or(CompassConfigurationError::ExpectedFieldWithType(
                (*key).into(),
                String::from("Array"),
            ))?
            .to_owned();
        Ok(array)
    }

    fn get_config_i64<A: Into<String>, B: Into<String>>(
        &self,
        key: &A,
        parent_key: &B,
    ) -> Result<i64, CompassConfigurationError> {
        let index: String = (*key).into();
        let value = self
            .get(index)
            .ok_or(CompassConfigurationError::ExpectedFieldForComponent(
                (*key).into(),
                (*parent_key).into(),
            ))?
            .as_i64()
            .ok_or(CompassConfigurationError::ExpectedFieldWithType(
                (*key).into(),
                String::from("64-bit signed integer"),
            ))?;
        Ok(value)
    }

    fn get_config_f64<A: Into<String>, B: Into<String>>(
        &self,
        key: &A,
        parent_key: &B,
    ) -> Result<f64, CompassConfigurationError> {
        let index: String = (*key).into();
        let value = self
            .get(index)
            .ok_or(CompassConfigurationError::ExpectedFieldForComponent(
                (*key).into(),
                (*parent_key).into(),
            ))?
            .as_f64()
            .ok_or(CompassConfigurationError::ExpectedFieldWithType(
                (*key).into(),
                String::from("64-bit floating point"),
            ))?;
        Ok(value)
    }

    fn get_config_from_str<A: Into<String>, B: Into<String>, T: FromStr>(
        &self,
        key: &A,
        parent_key: &B,
    ) -> Result<T, CompassConfigurationError> {
        let index: String = (*key).into();
        let value = self
            .get(index)
            .ok_or(CompassConfigurationError::ExpectedFieldForComponent(
                (*key).into(),
                (*parent_key).into(),
            ))?
            .as_str()
            .ok_or(CompassConfigurationError::ExpectedFieldWithType(
                (*key).into(),
                String::from("string-parseable"),
            ))?;
        let result = T::from_str(value).map_err(|_| {
            CompassConfigurationError::ExpectedFieldWithType(
                (*key).into(),
                format!("failed to parse type from string {}", value),
            )
        })?;
        Ok(result)
    }

    fn get_config_serde<A: Into<String>, B: Into<String>, T: de::DeserializeOwned>(
        &self,
        key: &A,
        parent_key: &B,
    ) -> Result<T, CompassConfigurationError> {
        let index: String = (*key).into();
        let value = self
            .get(index)
            .ok_or(CompassConfigurationError::ExpectedFieldForComponent(
                (*key).into(),
                (*parent_key).into(),
            ))?
            .to_owned();

        let result: T = serde_json::from_value(value)
            .map_err(CompassConfigurationError::SerdeDeserializationError)?;
        Ok(result)
    }
    fn get_config_serde_optional<A: Into<String>, B: Into<String>, T: de::DeserializeOwned>(
        &self,
        key: &A,
        _parent_key: &B,
    ) -> Result<Option<T>, CompassConfigurationError> {
        let index: String = (*key).into();
        match self.get(index) {
            None => Ok(None),
            Some(value) => {
                let result: T = serde_json::from_value(*value)
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
    fn normalize_file_paths<S: Into<String>>(
        &self,
        parent_key: &S,
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
                        .ok_or(CompassConfigurationError::FileNormalizationError(
                            path_string.clone(),
                        ))?
                        .to_string();
                    if new_path.is_file() {
                        Ok(serde_json::Value::String(new_path_string))
                    } else {
                        // if we can't find the file in either location, we throw an error
                        Err(CompassConfigurationError::FileNormalizationNotFound(
                            (*parent_key).into(),
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
                            (*key).into(),
                            value.normalize_file_paths(key, root_config_path)?,
                        );
                    } else {
                        new_obj.insert((*key).into(), value.clone());
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
