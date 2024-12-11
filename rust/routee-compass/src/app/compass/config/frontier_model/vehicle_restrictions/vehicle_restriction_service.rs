use super::{
    vehicle_parameters::VehicleParameters, vehicle_restriction::VehicleRestriction,
    vehicle_restriction_model::VehicleRestrictionFrontierModel,
};
use routee_compass_core::model::{
    frontier::{
        frontier_model::FrontierModel, frontier_model_error::FrontierModelError,
        frontier_model_service::FrontierModelService,
    },
    network::edge_id::EdgeId,
    state::state_model::StateModel,
};
use std::{collections::HashMap, sync::Arc};

#[derive(Clone)]
pub struct VehicleRestrictionFrontierService {
    pub vehicle_restriction_lookup: Arc<HashMap<EdgeId, Vec<VehicleRestriction>>>,
}

impl FrontierModelService for VehicleRestrictionFrontierService {
    fn build(
        &self,
        query: &serde_json::Value,
        _state_model: Arc<StateModel>,
    ) -> Result<Arc<dyn FrontierModel>, FrontierModelError> {
        let service: Arc<VehicleRestrictionFrontierService> = Arc::new(self.clone());

        let vehicle_parameters = VehicleParameters::from_query(query)?;

        let model = VehicleRestrictionFrontierModel {
            service,
            vehicle_parameters,
        };

        Ok(Arc::new(model))
    }
}
