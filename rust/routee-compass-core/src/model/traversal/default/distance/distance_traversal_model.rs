use uom::si::f64::Length;
use uom::ConstZero;

use crate::algorithm::search::SearchTree;
use crate::model::network::{Edge, Vertex};
use crate::model::state::StateModel;
use crate::model::state::StateVariable;
use crate::model::state::{InputFeature, StateVariableConfig};
use crate::model::traversal::default::fieldname;
use crate::model::traversal::traversal_model::TraversalModel;
use crate::model::traversal::traversal_model_error::TraversalModelError;
use crate::model::unit::DistanceUnit;
use crate::util::geo::haversine;

/// a model for traversing edges based on distance.
pub struct DistanceTraversalModel {
    pub distance_unit: DistanceUnit,
    pub include_trip_distance: bool,
    // Cached indices for performance
    edge_distance_idx: Option<usize>,
    trip_distance_idx: Option<usize>,
}

impl DistanceTraversalModel {
    pub fn new(distance_unit: DistanceUnit, include_trip_distance: bool) -> DistanceTraversalModel {
        Self {
            distance_unit,
            include_trip_distance,
            edge_distance_idx: None,
            trip_distance_idx: None,
        }
    }
}

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
        _tree: &SearchTree,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        let (_, edge, _) = trajectory;

        // Resolve indices (or use cached)
        let edge_distance_idx = match self.edge_distance_idx {
            Some(idx) => idx,
            None => state_model
                .get_index(fieldname::EDGE_DISTANCE)
                .map_err(|e| {
                    TraversalModelError::TraversalModelFailure(format!(
                        "Failed to find EDGE_DISTANCE index: {}",
                        e
                    ))
                })?,
        };

        state_model.add_distance_by_index(state, edge_distance_idx, &edge.distance)?;

        if self.include_trip_distance {
            let trip_distance_idx = match self.trip_distance_idx {
                Some(idx) => idx,
                None => state_model
                    .get_index(fieldname::TRIP_DISTANCE)
                    .map_err(|e| {
                        TraversalModelError::TraversalModelFailure(format!(
                            "Failed to find TRIP_DISTANCE index: {}",
                            e
                        ))
                    })?,
            };
            state_model.add_distance_by_index(state, trip_distance_idx, &edge.distance)?;
        }
        Ok(())
    }

    /// uses a haversine distance to estimate the distance between two vertices.
    fn estimate_traversal(
        &self,
        od: (&Vertex, &Vertex),
        state: &mut Vec<StateVariable>,
        _tree: &SearchTree,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        let (src, dst) = od;
        let distance =
            haversine::coord_distance(&src.coordinate, &dst.coordinate).map_err(|e| {
                TraversalModelError::TraversalModelFailure(format!(
                    "could not compute haversine distance between {src} and {dst}: {e}"
                ))
            })?;

        // Resolve indices (or use cached)
        let edge_distance_idx = match self.edge_distance_idx {
            Some(idx) => idx,
            None => state_model
                .get_index(fieldname::EDGE_DISTANCE)
                .map_err(|e| {
                    TraversalModelError::TraversalModelFailure(format!(
                        "Failed to find EDGE_DISTANCE index: {}",
                        e
                    ))
                })?,
        };

        if self.include_trip_distance {
            let trip_distance_idx = match self.trip_distance_idx {
                Some(idx) => idx,
                None => state_model
                    .get_index(fieldname::TRIP_DISTANCE)
                    .map_err(|e| {
                        TraversalModelError::TraversalModelFailure(format!(
                            "Failed to find TRIP_DISTANCE index: {}",
                            e
                        ))
                    })?,
            };
            state_model.add_distance_by_index(state, trip_distance_idx, &distance)?;
        }
        state_model.add_distance_by_index(state, edge_distance_idx, &distance)?;
        Ok(())
    }
}
