use crate::model::state::state_feature::StateFeature;
use crate::model::state::state_model::StateModel;
use crate::model::traversal::traversal_model::TraversalModel;
use crate::model::unit::as_f64::AsF64;
use crate::model::unit::DistanceUnit;
use crate::model::unit::BASE_DISTANCE_UNIT;
use crate::model::{
    property::{edge::Edge, vertex::Vertex},
    traversal::{state::state_variable::StateVar, traversal_model_error::TraversalModelError},
};
use crate::util::geo::haversine;
use std::sync::Arc;

/// A simple traversal model that uses the edge distance as the cost of traversal.
pub struct DistanceTraversalModel {
    state_model: Arc<StateModel>,
    distance_unit: DistanceUnit,
}

impl DistanceTraversalModel {
    pub fn new(
        state_model: Arc<StateModel>,
        distance_unit: DistanceUnit,
    ) -> DistanceTraversalModel {
        DistanceTraversalModel {
            state_model,
            distance_unit,
        }
    }
}

impl TraversalModel for DistanceTraversalModel {
    //
    fn traverse_edge(
        &self,
        _src: &Vertex,
        edge: &Edge,
        _dst: &Vertex,
        state: &mut Vec<StateVar>,
    ) -> Result<(), TraversalModelError> {
        let distance = BASE_DISTANCE_UNIT.convert(edge.distance, self.distance_unit);
        self.state_model
            .update_add(state, "distance", &StateVar(distance.as_f64()))?;
        Ok(())
    }

    fn access_edge(
        &self,
        _v1: &Vertex,
        _src: &Edge,
        _v2: &Vertex,
        _dst: &Edge,
        _v3: &Vertex,
        mut _state: &mut Vec<StateVar>,
    ) -> Result<(), TraversalModelError> {
        Ok(())
    }

    fn estimate_traversal(
        &self,
        src: &Vertex,
        dst: &Vertex,
        state: &mut Vec<StateVar>,
    ) -> Result<(), TraversalModelError> {
        let distance =
            haversine::coord_distance(&src.coordinate, &dst.coordinate, self.distance_unit)
                .map_err(TraversalModelError::NumericError)?;
        self.state_model
            .update_add(state, "distance", &StateVar(distance.as_f64()))?;
        Ok(())
    }

    /// no additional state features are needed
    fn state_features(&self) -> Vec<(String, StateFeature)> {
        vec![]
    }
}
