use super::{
    vehicle_parameters::VehicleParameters,
    vehicle_restriction_service::VehicleRestrictionFrontierService,
};
use routee_compass_core::model::{
    frontier::{frontier_model::FrontierModel, frontier_model_error::FrontierModelError},
    network::Edge,
    state::state_model::StateModel,
    traversal::StateVar,
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
        _tree: &std::collections::HashMap<
            routee_compass_core::model::network::VertexId,
            routee_compass_core::algorithm::search::search_tree_branch::SearchTreeBranch,
        >,
        _direction: &routee_compass_core::algorithm::search::direction::Direction,
        _state_model: &StateModel,
    ) -> Result<bool, FrontierModelError> {
        self.valid_edge(edge)
    }

    fn valid_edge(&self, edge: &Edge) -> Result<bool, FrontierModelError> {
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
