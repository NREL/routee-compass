use super::compass_input_field::CompassInputField;
use crate::app::compass::compass_app_error::CompassAppError;

pub trait CompassJsonExtensions {
    fn get_queries(&self) -> Result<Vec<serde_json::Value>, CompassAppError>;
}

impl CompassJsonExtensions for serde_json::Value {
    /// attempts to read queries from the user in the three following ways:
    ///   1. top-level of JSON is an array -> return it directly
    ///   2. top-level of JSON is object without a "queries" field -> wrap it in an array and return it
    ///   3. top-level of JSON is object with a "queries" field ->  if the value at "queries" is an array, return it
    fn get_queries(&self) -> Result<Vec<serde_json::Value>, CompassAppError> {
        match self {
            serde_json::Value::Array(queries) => Ok(queries.to_owned()),
            serde_json::Value::Object(obj) => match obj.get(CompassInputField::Queries.to_str()) {
                None => Ok(vec![self.to_owned()]),
                Some(value) => match value {
                    serde_json::Value::Array(vec) => Ok(vec.to_owned()),
                    _ => {
                        let msg = String::from("user JSON argument must be an object or an array");
                        Err(CompassAppError::InvalidInput(msg))
                    }
                },
            },
            _ => Err(CompassAppError::InvalidInput(String::from(
                "expected object, object with queries, or array input",
            ))),
        }
    }
}
