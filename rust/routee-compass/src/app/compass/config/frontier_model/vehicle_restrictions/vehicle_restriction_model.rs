use super::{
    vehicle_parameters::VehicleParameters,
    vehicle_restriction_service::VehicleRestrictionFrontierService,
};
use routee_compass_core::model::{
    frontier::{frontier_model::FrontierModel, frontier_model_error::FrontierModelError},
    graph::Edge,
    state::state_model::StateModel,
    traversal::state::state_variable::StateVar,
};
use std::sync::Arc;

pub struct VehicleRestrictionFrontierModel {
    pub service: Arc<VehicleRestrictionFrontierService>,
    pub vehicle_parameters: VehicleParameters,
}

impl FrontierModel for VehicleRestrictionFrontierModel {
    fn valid_frontier(
        &self,
        edge: &Edge,
        _state: &[StateVar],
        _previous_edge: Option<&Edge>,
        _state_model: &StateModel,
    ) -> Result<bool, FrontierModelError> {
        match self.service.vehicle_restriction_lookup.get(&edge.edge_id) {
            None => Ok(true),
            Some(vehicle_restrictions) => {
                for restriction in vehicle_restrictions.iter() {
                    if !restriction.valid(&self.vehicle_parameters) {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
        }
    }
}
