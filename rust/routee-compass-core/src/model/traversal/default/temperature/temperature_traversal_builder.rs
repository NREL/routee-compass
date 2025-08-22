use crate::model::traversal::{
    default::temperature::TemperatureTraversalService, TraversalModelBuilder, TraversalModelError,
    TraversalModelService,
};
use std::sync::Arc;

pub struct TemperatureTraversalBuilder {}

impl TraversalModelBuilder for TemperatureTraversalBuilder {
    fn build(
        &self,
        _parameters: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModelService>, TraversalModelError> {
        let service = Arc::new(TemperatureTraversalService {});
        Ok(service)
    }
}
