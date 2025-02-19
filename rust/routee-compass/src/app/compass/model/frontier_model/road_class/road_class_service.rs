use super::road_class_model::RoadClassFrontierModel;
use routee_compass_core::model::{
    frontier::{FrontierModel, FrontierModelError, FrontierModelService},
    state::StateModel,
};
use serde_json::Value;
use std::{collections::HashSet, sync::Arc};

#[derive(Clone)]
pub struct RoadClassFrontierService {
    pub road_class_by_edge: Arc<Box<[String]>>,
}

impl FrontierModelService for RoadClassFrontierService {
    fn build(
        &self,
        query: &serde_json::Value,
        _state_model: Arc<StateModel>,
    ) -> Result<Arc<dyn FrontierModel>, FrontierModelError> {
        let query_road_classes = match query.get("road_classes").map(read_road_classes_from_query) {
            Some(Err(e)) => Err(e),
            Some(Ok(road_classes)) => Ok(Some(road_classes)),
            None => Ok(None),
        }?;

        let service: Arc<RoadClassFrontierService> = Arc::new(self.clone());
        let model = RoadClassFrontierModel {
            service,
            query_road_classes,
        };
        Ok(Arc::new(model))
    }
}

/// decodes the query `road_classes` value into a set of road class identifiers
fn read_road_classes_from_query(value: &Value) -> Result<HashSet<String>, FrontierModelError> {
    let arr = value.as_array().ok_or_else(|| {
        FrontierModelError::BuildError(format!(
            "query 'road_classes' value must be an array, found '{}'",
            value
        ))
    })?;
    let arr_str = arr
        .iter()
        .enumerate()
        .map(|(idx, c)| {
            c.as_str()
                .ok_or_else(|| {
                    FrontierModelError::BuildError(format!(
                        "query 'road_classes[{}]' value must be a string, found '{}'",
                        idx, c
                    ))
                })
                .map(String::from)
        })
        .collect::<Result<HashSet<_>, _>>()?;

    Ok(arr_str)
}
