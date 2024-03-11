use crate::model::state::state_feature::StateFeature;
use crate::model::state::state_model::StateModel;
use crate::model::traversal::traversal_model::TraversalModel;
use crate::model::unit::DistanceUnit;
use crate::model::unit::BASE_DISTANCE_UNIT;
use crate::model::{
    property::{edge::Edge, vertex::Vertex},
    traversal::{state::state_variable::StateVar, traversal_model_error::TraversalModelError},
};
use crate::util::geo::haversine;

/// A simple traversal model that uses the edge distance as the cost of traversal.
pub struct DistanceTraversalModel {
    distance_unit: DistanceUnit,
}

impl DistanceTraversalModel {
    pub fn new(distance_unit: DistanceUnit) -> DistanceTraversalModel {
        DistanceTraversalModel { distance_unit }
    }
}

impl TraversalModel for DistanceTraversalModel {
    //
    fn traverse_edge(
        &self,
        trajectory: (&Vertex, &Edge, &Vertex),
        state: &mut Vec<StateVar>,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        let (_, edge, _) = trajectory;
        let distance = BASE_DISTANCE_UNIT.convert(&edge.distance, &self.distance_unit);
        state_model.add_distance(state, "distance", &distance, &self.distance_unit)?;
        Ok(())
    }

    fn access_edge(
        &self,
        _trajectory: (&Vertex, &Edge, &Vertex, &Edge, &Vertex),
        _state: &mut Vec<StateVar>,
        _state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        Ok(())
    }

    fn estimate_traversal(
        &self,
        od: (&Vertex, &Vertex),
        state: &mut Vec<StateVar>,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        let (src, dst) = od;
        let distance =
            haversine::coord_distance(&src.coordinate, &dst.coordinate, self.distance_unit)
                .map_err(TraversalModelError::NumericError)?;
        state_model.add_distance(state, "distance", &distance, &self.distance_unit)?;
        Ok(())
    }

    /// no additional state features are needed
    fn state_features(&self) -> Vec<(String, StateFeature)> {
        vec![]
    }
}
