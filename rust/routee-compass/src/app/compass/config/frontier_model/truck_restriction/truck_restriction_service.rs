use super::{
    truck_parameters::TruckParameters, truck_restriction::TruckRestriction,
    truck_restriction_model::TruckRestrictionFrontierModel,
};
use routee_compass_core::model::{
    frontier::{
        frontier_model::FrontierModel, frontier_model_error::FrontierModelError,
        frontier_model_service::FrontierModelService,
    },
    road_network::edge_id::EdgeId,
    state::state_model::StateModel,
};
use std::{collections::HashMap, sync::Arc};

#[derive(Clone)]
pub struct TruckRestrictionFrontierService {
    pub truck_restriction_lookup: Arc<HashMap<EdgeId, Vec<TruckRestriction>>>,
}

impl FrontierModelService for TruckRestrictionFrontierService {
    fn build(
        &self,
        query: &serde_json::Value,
        _state_model: Arc<StateModel>,
    ) -> Result<Arc<dyn FrontierModel>, FrontierModelError> {
        let service: Arc<TruckRestrictionFrontierService> = Arc::new(self.clone());

        let truck_parameters = TruckParameters::from_query(query)?;

        let model = TruckRestrictionFrontierModel {
            service,
            truck_parameters,
        };

        Ok(Arc::new(model))
    }
}
