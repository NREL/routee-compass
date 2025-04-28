use std::borrow::Cow;

use crate::model::network::{Edge, Vertex};
use crate::model::state::StateFeature;
use crate::model::state::StateModel;
use crate::model::state::StateVariable;
use crate::model::traversal::traversal_model::TraversalModel;
use crate::model::traversal::traversal_model_error::TraversalModelError;
use crate::model::unit::{baseunit, Convert, DistanceUnit};
use crate::util::geo::haversine;

/// A simple traversal model that uses the edge distance as the cost of traversal.
pub struct DistanceTraversalModel {
    distance_unit: DistanceUnit,
}

impl DistanceTraversalModel {
    pub fn new(distance_unit: DistanceUnit) -> DistanceTraversalModel {
        DistanceTraversalModel { distance_unit }
    }
    const DISTANCE: &'static str = "distance";
}

impl TraversalModel for DistanceTraversalModel {
    //
    fn traverse_edge(
        &self,
        trajectory: (&Vertex, &Edge, &Vertex),
        state: &mut Vec<StateVariable>,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        let (_, edge, _) = trajectory;
        let mut distance = Cow::Borrowed(&edge.distance);
        baseunit::DISTANCE_UNIT.convert(&mut distance, &self.distance_unit)?;
        state_model.add_distance(
            state,
            &Self::DISTANCE.into(),
            &distance,
            &self.distance_unit,
        )?;
        Ok(())
    }

    fn estimate_traversal(
        &self,
        od: (&Vertex, &Vertex),
        state: &mut Vec<StateVariable>,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        let (src, dst) = od;
        let distance =
            haversine::coord_distance(&src.coordinate, &dst.coordinate, self.distance_unit)
                .map_err(|e| {
                    TraversalModelError::TraversalModelFailure(format!(
                        "could not compute haversine distance between {} and {}: {}",
                        src, dst, e
                    ))
                })?;
        state_model.add_distance(
            state,
            &Self::DISTANCE.into(),
            &distance,
            &self.distance_unit,
        )?;
        Ok(())
    }

    /// no additional state features are needed
    fn state_features(&self) -> Vec<(String, StateFeature)> {
        vec![]
    }
}
