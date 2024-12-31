use super::{road_class_model::RoadClassFrontierModel, road_class_parser::RoadClassParser};
use routee_compass_core::model::{
    frontier::{
        frontier_model::FrontierModel, frontier_model_error::FrontierModelError,
        frontier_model_service::FrontierModelService,
    },
    state::StateModel,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct RoadClassFrontierService {
    pub road_class_lookup: Arc<Box<[u8]>>,
    pub road_class_parser: RoadClassParser,
}

impl FrontierModelService for RoadClassFrontierService {
    fn build(
        &self,
        query: &serde_json::Value,
        _state_model: Arc<StateModel>,
    ) -> Result<Arc<dyn FrontierModel>, FrontierModelError> {
        let service: Arc<RoadClassFrontierService> = Arc::new(self.clone());
        let road_classes = self.road_class_parser.read_query(query).map_err(|e| {
            FrontierModelError::BuildError(format!(
                "Unable to parse incoming query road_classes due to: {}",
                e
            ))
        })?;
        let model = RoadClassFrontierModel {
            service,
            road_classes,
        };
        Ok(Arc::new(model))
    }
}
