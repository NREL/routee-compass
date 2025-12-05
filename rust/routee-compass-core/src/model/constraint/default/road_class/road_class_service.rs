use super::road_class_model::RoadClassConstraintModel;
use crate::model::{
    constraint::{ConstraintModel, ConstraintModelError, ConstraintModelService},
    state::StateModel,
};
use serde_json::Value;
use std::{collections::HashSet, sync::Arc};

#[derive(Clone)]
pub struct RoadClassFrontierService {
    pub road_class_by_edge: Arc<Box<[String]>>,
}

impl ConstraintModelService for RoadClassFrontierService {
    fn build(
        &self,
        query: &serde_json::Value,
        _state_model: Arc<StateModel>,
    ) -> Result<Arc<dyn ConstraintModel>, ConstraintModelError> {
        let query_road_classes = match query.get("road_classes").map(read_road_classes_from_query) {
            Some(Err(e)) => Err(e),
            Some(Ok(road_classes)) => Ok(Some(road_classes)),
            None => Ok(None),
        }?;

        let service: Arc<RoadClassFrontierService> = Arc::new(self.clone());
        let model = RoadClassConstraintModel {
            service,
            query_road_classes,
        };
        Ok(Arc::new(model))
    }
}

/// decodes the query `road_classes` value into a set of road class identifiers
fn read_road_classes_from_query(value: &Value) -> Result<HashSet<String>, ConstraintModelError> {
    let arr = value.as_array().ok_or_else(|| {
        ConstraintModelError::BuildError(format!(
            "query 'road_classes' value must be an array, found '{value}'"
        ))
    })?;
    // if the value is a string (or number or bool), store it as a valid road class
    let arr_str = arr
        .iter()
        .enumerate()
        .map(|(idx, c)| match c {
            Value::Bool(b) => Ok(b.to_string()),
            Value::Number(number) => Ok(number.to_string()),
            Value::String(string) => Ok(string.clone()),
            _ => Err(ConstraintModelError::BuildError(format!(
                "query 'road_classes[{idx}]' value must be a string, found '{c}'"
            ))),
        })
        .collect::<Result<HashSet<_>, _>>()?;

    Ok(arr_str)
}
