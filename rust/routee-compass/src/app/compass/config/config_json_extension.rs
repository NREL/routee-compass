use super::compass_configuration_error::CompassConfigurationError;
use serde::de;
use std::{path::PathBuf, str::FromStr};

pub const CONFIG_DIRECTORY_KEY: &str = "config_directory";

pub trait ConfigJsonExtensions {
    fn set_config_directory(
        &mut self,
        config_directory: String,
    ) -> Result<(), CompassConfigurationError>;
    fn get_config_path(
        &self,
        key: String,
        parent_key: String,
    ) -> Result<PathBuf, CompassConfigurationError>;
    fn get_config_path_optional(
        &self,
        key: String,
        parent_key: String,
    ) -> Result<Option<PathBuf>, CompassConfigurationError>;
    fn get_config_string(
        &self,
        key: String,
        parent_key: String,
    ) -> Result<String, CompassConfigurationError>;
    fn get_config_i64(
        &self,
        key: String,
        parent_key: String,
    ) -> Result<i64, CompassConfigurationError>;
    fn get_config_f64(
        &self,
        key: String,
        parent_key: String,
    ) -> Result<f64, CompassConfigurationError>;
    fn get_config_from_str<T: FromStr>(
        &self,
        key: String,
        parent_key: String,
    ) -> Result<T, CompassConfigurationError>;
    fn get_config_serde<T: de::DeserializeOwned>(
        &self,
        key: String,
        parent_key: String,
    ) -> Result<T, CompassConfigurationError>;
    fn get_config_serde_optional<T: de::DeserializeOwned>(
        &self,
        key: String,
        parent_key: String,
    ) -> Result<Option<T>, CompassConfigurationError>;
}

impl ConfigJsonExtensions for serde_json::Value {
    fn set_config_directory(
        &mut self,
        config_directory: String,
    ) -> Result<(), CompassConfigurationError> {
        self.as_object_mut()
            .ok_or(CompassConfigurationError::InsertError(
                "Attempted to set config directory but the config is not a JSON object".to_string(),
            ))?
            .insert(
                CONFIG_DIRECTORY_KEY.to_string(),
                serde_json::Value::String(config_directory),
            )
            .ok_or(CompassConfigurationError::InsertError(format!(
                "Attempted to set config directory but the key {} already exists",
                CONFIG_DIRECTORY_KEY.to_string()
            )))?;
        Ok(())
    }
    fn get_config_path_optional(
        &self,
        key: String,
        parent_key: String,
    ) -> Result<Option<PathBuf>, CompassConfigurationError> {
        match self.get(key.clone()) {
            None => Ok(None),
            Some(value) => {
                let config_path = self.get_config_path(key, parent_key)?;
                Ok(Some(config_path))
            }
        }
    }
    fn get_config_path(
        &self,
        key: String,
        parent_key: String,
    ) -> Result<PathBuf, CompassConfigurationError> {
        let config_path_string = self
            .get(CONFIG_DIRECTORY_KEY)
            .ok_or(CompassConfigurationError::ExpectedFieldForComponent(
                CONFIG_DIRECTORY_KEY.to_string(),
                parent_key.clone(),
            ))?
            .as_str()
            .ok_or(CompassConfigurationError::ExpectedFieldWithType(
                key.clone(),
                String::from("String"),
            ))?;

        let config_path = PathBuf::from(config_path_string);

        let path_string = self.get_config_string(key.clone(), parent_key.clone())?;
        let path = PathBuf::from(path_string.clone());

        // if file can be found, just return it
        if path.is_file() {
            return Ok(path);
        }

        // try searching in the config directory
        let path_from_config = config_path.join(path);
        if path_from_config.is_file() {
            return Ok(path_from_config);
        }

        // can't find the file
        Err(CompassConfigurationError::FileNotFoundForComponent(
            path_string,
            key,
            parent_key,
        ))
    }
    fn get_config_string(
        &self,
        key: String,
        parent_key: String,
    ) -> Result<String, CompassConfigurationError> {
        let value = self
            .get(&key)
            .ok_or(CompassConfigurationError::ExpectedFieldForComponent(
                key.clone(),
                parent_key.clone(),
            ))?
            .as_str()
            .map(String::from)
            .ok_or(CompassConfigurationError::ExpectedFieldWithType(
                key.clone(),
                String::from("String"),
            ))?;
        return Ok(value);
    }

    fn get_config_i64(
        &self,
        key: String,
        parent_key: String,
    ) -> Result<i64, CompassConfigurationError> {
        let value = self
            .get(&key)
            .ok_or(CompassConfigurationError::ExpectedFieldForComponent(
                key.clone(),
                parent_key.clone(),
            ))?
            .as_i64()
            .ok_or(CompassConfigurationError::ExpectedFieldWithType(
                key.clone(),
                String::from("64-bit signed integer"),
            ))?;
        return Ok(value);
    }

    fn get_config_f64(
        &self,
        key: String,
        parent_key: String,
    ) -> Result<f64, CompassConfigurationError> {
        let value = self
            .get(&key)
            .ok_or(CompassConfigurationError::ExpectedFieldForComponent(
                key.clone(),
                parent_key.clone(),
            ))?
            .as_f64()
            .ok_or(CompassConfigurationError::ExpectedFieldWithType(
                key.clone(),
                String::from("64-bit floating point"),
            ))?;
        return Ok(value);
    }

    fn get_config_from_str<T: FromStr>(
        &self,
        key: String,
        parent_key: String,
    ) -> Result<T, CompassConfigurationError> {
        let value = self
            .get(&key)
            .ok_or(CompassConfigurationError::ExpectedFieldForComponent(
                key.clone(),
                parent_key.clone(),
            ))?
            .as_str()
            .ok_or(CompassConfigurationError::ExpectedFieldWithType(
                key.clone(),
                String::from("string-parseable"),
            ))?;
        let result = T::from_str(value).map_err(|_| {
            CompassConfigurationError::ExpectedFieldWithType(
                key.clone(),
                format!("failed to parse type from string {}", value),
            )
        })?;
        return Ok(result);
    }

    fn get_config_serde<T: de::DeserializeOwned>(
        &self,
        key: String,
        parent_key: String,
    ) -> Result<T, CompassConfigurationError> {
        let value = self
            .get(key.clone())
            .ok_or(CompassConfigurationError::ExpectedFieldForComponent(
                key.clone(),
                parent_key.clone(),
            ))?
            .to_owned();

        let result: T = serde_json::from_value(value).map_err(|_| {
            CompassConfigurationError::ExpectedFieldWithType(
                key.clone(),
                String::from("string-parseable"),
            )
        })?;
        return Ok(result);
    }
    fn get_config_serde_optional<T: de::DeserializeOwned>(
        &self,
        key: String,
        _parent_key: String,
    ) -> Result<Option<T>, CompassConfigurationError> {
        match self.get(key.clone()) {
            None => Ok(None),
            Some(value) => {
                let result: T = serde_json::from_value(value.clone())
                    .map_err(CompassConfigurationError::SerdeDeserializationError)?;
                return Ok(Some(result));
            }
        }
    }
}
