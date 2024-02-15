use super::road_class_model::RoadClassFrontierModel;
use crate::app::compass::config::config_json_extension::ConfigJsonExtensions;
use routee_compass_core::model::frontier::{
    frontier_model::FrontierModel, frontier_model_error::FrontierModelError,
    frontier_model_service::FrontierModelService,
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

#[derive(Clone)]
pub struct RoadClassFrontierService {
    pub road_class_lookup: Arc<Box<[u8]>>,

    // Extension point for providing a mapping from a string based road class (like OSM classicfication) to a u8
    pub road_class_mapping: Option<HashMap<String, u8>>,
}

impl FrontierModelService for RoadClassFrontierService {
    fn build(
        &self,
        query: &serde_json::Value,
    ) -> Result<Arc<dyn FrontierModel>, FrontierModelError> {
        let service: Arc<RoadClassFrontierService> = Arc::new(self.clone());
        let road_classes = query
            .get_config_serde_optional::<HashSet<u8>>(&"road_classes", &"")
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
