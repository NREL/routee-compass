use crate::model::state::state_model::StateModel;
use crate::model::traversal::default::distance_traversal_model::DistanceTraversalModel;
use crate::model::traversal::traversal_model::TraversalModel;
use crate::model::traversal::traversal_model_error::TraversalModelError;
use crate::model::traversal::traversal_model_service::TraversalModelService;
use crate::model::unit::DistanceUnit;
use std::sync::Arc;

pub struct DistanceTraversalService {
    pub distance_unit: DistanceUnit,
}

impl TraversalModelService for DistanceTraversalService {
    fn build(
        &self,
        _parameters: &serde_json::Value,
        state_model: Arc<StateModel>,
    ) -> Result<Arc<dyn TraversalModel>, TraversalModelError> {
        let m: Arc<dyn TraversalModel> = Arc::new(DistanceTraversalModel::new(
            state_model.clone(),
            self.distance_unit,
        ));
        Ok(m)
    }
}
