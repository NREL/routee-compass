use std::borrow::Cow;

use crate::model::network::{Edge, Vertex};
use crate::model::state::StateFeature;
use crate::model::state::StateModel;
use crate::model::state::StateVariable;
use crate::model::traversal::traversal_model::TraversalModel;
use crate::model::traversal::traversal_model_error::TraversalModelError;
use crate::model::unit::{baseunit, Convert, Distance, DistanceUnit};
use crate::util::geo::haversine;

/// a model for traversing edges based on distance.
pub struct DistanceTraversalModel {
    /// this is the unit used to store distance in the state vector
    distance_unit: DistanceUnit,
}

impl DistanceTraversalModel {
    pub fn new(distance_unit: DistanceUnit) -> DistanceTraversalModel {
        DistanceTraversalModel { distance_unit }
    }
    const TRIP_DISTANCE: &'static str = "trip_distance";
    const LEG_DISTANCE: &'static str = "leg_distance";
}

impl TraversalModel for DistanceTraversalModel {
    /// traverses a graph edge and updates the state vector with the distance.
    /// the distance values are directly available on the [`Graph`] model edges.
    ///
    /// [Graph]: crate::model::network::Graph
    fn traverse_edge(
        &self,
        trajectory: (&Vertex, &Edge, &Vertex),
        state: &mut Vec<StateVariable>,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        let (_, edge, _) = trajectory;
        let mut distance = Cow::Borrowed(&edge.distance);
        baseunit::DISTANCE_UNIT.convert(&mut distance, &self.distance_unit)?;
        state_model.add_distance(state, Self::TRIP_DISTANCE, &distance, &self.distance_unit)?;
        Ok(())
    }

    /// uses a haversine distance to estimate the distance between two vertices.
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
        state_model.add_distance(state, Self::TRIP_DISTANCE, &distance, &self.distance_unit)?;
        state_model.set_distance(state, Self::LEG_DISTANCE, &distance, &self.distance_unit)?;
        Ok(())
    }

    fn input_features(&self) -> Vec<(String, StateFeature)> {
        vec![]
    }

    fn output_features(&self) -> Vec<(String, StateFeature)> {
        vec![
            (
                String::from(Self::TRIP_DISTANCE),
                StateFeature::Distance {
                    distance_unit: self.distance_unit,
                    initial: Distance::ZERO,
                },
            ),
            (
                String::from(Self::LEG_DISTANCE),
                StateFeature::Distance {
                    distance_unit: self.distance_unit,
                    initial: Distance::ZERO,
                },
            ),
        ]
    }
}
