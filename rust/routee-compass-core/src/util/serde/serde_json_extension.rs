use serde::de::Error;
use serde_json::Value::{self, Object};
use std::any::Any;

pub trait SerdeJsonExtension {
    fn merge(&self, that: &serde_json::Value) -> Result<Value, serde_json::Error>;
}

impl SerdeJsonExtension for Value {
    fn merge(&self, that: &serde_json::Value) -> Result<Value, serde_json::Error> {
        match (self.clone(), that) {
            (Object(ref mut t), Object(ref e)) => {
                for (k, v) in e {
                    t.insert(k.clone(), v.clone());
                }
                Ok(serde_json::json!(t))
            }
            (Object(_), b) => Err(Error::custom(format!(
                "merging this JSON object but 'that' instance is not an object, it is a {:?}",
                b.type_id()
            ))),
            (a, Object(_)) => Err(Error::custom(format!(
                "merging 'that' JSON object but 'this' (lhs) instance is not an object, it is a {:?}",
                a.type_id()
            ))),
            _ => Err(Error::custom(String::from(
                "merging two JSONs but neither are 'object' types (and must be)"
            ))),
        }
    }
}
