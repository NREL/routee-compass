use super::compass_input_field::CompassInputField;
use crate::app::app_error::AppError;

pub trait CompassJsonExtensions {
    fn get_queries(&self) -> Result<Vec<serde_json::Value>, AppError>;
}

impl CompassJsonExtensions for serde_json::Value {
    /// attempts to grab a vector of queries from the Queries field. if there is none,
    /// then treat the entire input as a query and return it wrapped in a vector.
    fn get_queries(&self) -> Result<Vec<serde_json::Value>, AppError> {
        match self.get(CompassInputField::Queries.to_str()) {
            None => Ok(vec![self.to_owned()]),
            Some(value) => match value {
                serde_json::Value::Array(vec) => Ok(vec.to_owned()),
                _ => {
                    let msg = String::from("user JSON argument must be an object or an array");
                    Err(AppError::InvalidInput(msg))
                }
            },
        }
    }
}
