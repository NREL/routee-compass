use super::{
    vehicle_parameters::VehicleParameters,
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
    pub vehicle_parameters: VehicleParameters,
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
                for restriction in vehicle_restrictions.iter() {
                    let valid_edge = restriction.valid(&self.vehicle_parameters).map_err(|e| {
                        FrontierModelError::FrontierModelError(format!(
                            "failed testing edge validity in frontier model due to: {}",
                            e
                        ))
                    })?;
                    if !valid_edge {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
        }
    }
}
