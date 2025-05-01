use super::{
    vehicle_parameters::VehicleParameter,
    vehicle_restriction_service::VehicleRestrictionFrontierService,
};
use routee_compass_core::model::{
    frontier::{FrontierModel, FrontierModelError},
    network::Edge,
    state::{StateModel, StateVariable},
};
use std::sync::Arc;

pub struct VehicleRestrictionFrontierModel {
    pub service: Arc<VehicleRestrictionFrontierService>,
    pub vehicle_parameters: Vec<VehicleParameter>,
}

impl FrontierModel for VehicleRestrictionFrontierModel {
    fn valid_frontier(
        &self,
        edge: &Edge,
        _state: &[StateVariable],
        _tree: &std::collections::HashMap<
            routee_compass_core::model::network::VertexId,
            routee_compass_core::algorithm::search::SearchTreeBranch,
        >,
        _direction: &routee_compass_core::algorithm::search::Direction,
        _state_model: &StateModel,
    ) -> Result<bool, FrontierModelError> {
        self.valid_edge(edge)
    }

    fn valid_edge(&self, edge: &Edge) -> Result<bool, FrontierModelError> {
        match self.service.vehicle_restriction_lookup.get(&edge.edge_id) {
            None => Ok(true),
            Some(vehicle_restrictions) => {
                let valid = self.vehicle_parameters.iter().all(|vehicle_parameter| {
                    match vehicle_restrictions.get(&vehicle_parameter.name()) {
                        Some(restriction) => vehicle_parameter <= restriction,
                        None => true,
                    }
                });
                Ok(valid)
            }
        }
    }
}
