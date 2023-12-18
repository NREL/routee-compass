use super::road_class_model::RoadClassFrontierModel;
use crate::app::compass::config::config_json_extension::ConfigJsonExtensions;
use routee_compass_core::model::frontier::{
    frontier_model::FrontierModel, frontier_model_error::FrontierModelError,
    frontier_model_service::FrontierModelService,
};
use std::{collections::HashSet, sync::Arc};

#[derive(Clone)]
pub struct RoadClassFrontierService {
    pub road_class_lookup: Arc<Box<[String]>>,
}

impl FrontierModelService for RoadClassFrontierService {
    fn build(
        &self,
        query: &serde_json::Value,
    ) -> Result<Arc<dyn FrontierModel>, FrontierModelError> {
        let service: Arc<RoadClassFrontierService> = Arc::new(self.clone());
        let road_classes = query
            .get_config_serde_optional::<HashSet<String>>(&"road_classes", &"")
            .map_err(|e| {
                FrontierModelError::BuildError(format!("unable to deserialize as array: {}", e))
            })?;
        let model = RoadClassFrontierModel {
            service,
            road_classes,
        };
        Ok(Arc::new(model))
    }
}
