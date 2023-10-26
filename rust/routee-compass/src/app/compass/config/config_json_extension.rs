use super::compass_configuration_error::CompassConfigurationError;
use serde::de;
use std::str::FromStr;

pub trait ConfigJsonExtensions {
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
