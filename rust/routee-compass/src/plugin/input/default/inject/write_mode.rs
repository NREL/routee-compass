use crate::plugin::input::InputPluginError;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum WriteMode {
    Overwrite,
    Append,
    FailIfExisting,
    Ignore,
}

impl WriteMode {
    /// write a key/value pair to an existing query using this [`WriteMode`].
    pub fn write_to_query(
        &self,
        query: &mut serde_json::Value,
        key: &str,
        value: &Value,
    ) -> Result<(), InputPluginError> {
        match self {
            WriteMode::Overwrite => {
                query[key] = value.clone();
                Ok(())
            }
            WriteMode::Append => match query.get_mut(key) {
                Some(existing) => match (existing, value) {
                    (Value::Array(a), Value::Array(b)) => {
                        a.append(&mut b.clone());
                        Ok(())
                    }
                    (Value::Object(a), Value::Object(b)) => {
                        for (k, v) in b.iter() {
                            a.insert(k.to_string(), v.clone());
                        }
                        Ok(())
                    }
                    (a, b) => Err(InputPluginError::InputPluginFailed(format!(
                        "while injecting value to key '{}' on append mode, found that types mismatch ({} != {})",
                        key,
                        json_type(a),
                        json_type(b)
                    ))),
                },
                None => {
                    query[key] = value.clone();
                    Ok(())
                }
            },
            WriteMode::FailIfExisting => {
                match query.get(key) {
                    Some(existing) => {
                        let msg = format!("while injecting value '{}' with fail_if_existing mode, found key already exists with value: {}", 
                            key,
                            serde_json::to_string(existing).unwrap_or_default()
                        );
                        Err(InputPluginError::InputPluginFailed(msg))
                    },
                    None => {
                        Self::Overwrite.write_to_query(query, key, value)
                    },
                }
            },
            WriteMode::Ignore => {
                match query.get(key) {
                    Some(_) => {
                        Ok(())
                    },
                    None => {
                        Self::Overwrite.write_to_query(query, key, value)
                    },
                }
            },
        }
    }
}

fn json_type(a: &Value) -> String {
    match a {
        Value::Null => String::from("Null"),
        Value::Bool(_) => String::from("Bool"),
        Value::Number(_) => String::from("Number"),
        Value::String(_) => String::from("String"),
        Value::Array(_) => String::from("Array"),
        Value::Object(_) => String::from("Object"),
    }
}
