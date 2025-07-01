use uom::si::f64::Length;
use uom::ConstZero;

use crate::model::network::{Edge, Vertex};
use crate::model::state::StateModel;
use crate::model::state::StateVariable;
use crate::model::state::{InputFeature, StateFeature};
use crate::model::traversal::default::fieldname;
use crate::model::traversal::traversal_model::TraversalModel;
use crate::model::traversal::traversal_model_error::TraversalModelError;
use crate::model::unit::DistanceUnit;
use crate::util::geo::haversine;

/// a model for traversing edges based on distance.
pub struct DistanceTraversalModel {}

impl TraversalModel for DistanceTraversalModel {
    fn name(&self) -> String {
        String::from("Distance Traversal Model")
    }
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

        state_model.set_distance(state, fieldname::EDGE_DISTANCE, &edge.distance)?;
        state_model.add_distance(state, fieldname::TRIP_DISTANCE, &edge.distance)?;
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
            haversine::coord_distance(&src.coordinate, &dst.coordinate).map_err(|e| {
                TraversalModelError::TraversalModelFailure(format!(
                    "could not compute haversine distance between {} and {}: {}",
                    src, dst, e
                ))
            })?;
        state_model.add_distance(state, fieldname::TRIP_DISTANCE, &distance)?;
        state_model.set_distance(state, fieldname::EDGE_DISTANCE, &distance)?;
        Ok(())
    }

    fn input_features(&self) -> Vec<InputFeature> {
        vec![]
    }

    fn output_features(&self) -> Vec<(String, StateFeature)> {
        vec![
            (
                String::from(fieldname::TRIP_DISTANCE),
                StateFeature::Distance {
                    value: Length::ZERO,
                    accumulator: true,
                    output_unit: Some(DistanceUnit::default()),
                },
            ),
            (
                String::from(fieldname::EDGE_DISTANCE),
                StateFeature::Distance {
                    value: Length::ZERO,
                    accumulator: false,
                    output_unit: Some(DistanceUnit::default()),
                },
            ),
        ]
    }
}
