use super::{
    truck_parameters::TruckParameters, truck_restriction_service::TruckRestrictionFrontierService,
};
use routee_compass_core::model::{
    frontier::{frontier_model::FrontierModel, frontier_model_error::FrontierModelError},
    property::edge::Edge,
    state::state_model::StateModel,
    traversal::state::state_variable::StateVar,
};
use std::sync::Arc;

pub struct TruckRestrictionFrontierModel {
    pub service: Arc<TruckRestrictionFrontierService>,
    pub truck_parameters: TruckParameters,
}

impl FrontierModel for TruckRestrictionFrontierModel {
    fn valid_frontier(
        &self,
        edge: &Edge,
        _state: &[StateVar],
        _previous_edge: Option<&Edge>,
        _state_model: &StateModel,
    ) -> Result<bool, FrontierModelError> {
        match self.service.truck_restriction_lookup.get(&edge.edge_id) {
            None => Ok(true),
            Some(truck_restrictions) => {
                for restriction in truck_restrictions.iter() {
                    if !restriction.valid(&self.truck_parameters) {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
        }
    }
}
