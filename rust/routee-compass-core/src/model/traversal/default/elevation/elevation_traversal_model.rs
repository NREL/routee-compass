use std::sync::Arc;

use uom::{
    si::f64::{Length, Ratio},
    ConstZero,
};

use crate::model::{
    network::{Edge, Vertex},
    state::{StateFeature, StateModel, StateVariable},
    traversal::{default::fieldname, TraversalModel, TraversalModelError, TraversalModelService},
};

use super::elevation_change::ElevationChange;

#[derive(Clone, Debug)]
pub struct ElevationTraversalModel {}

impl TraversalModelService for ElevationTraversalModel {
    fn build(
        &self,
        _query: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModel>, TraversalModelError> {
        Ok(Arc::new(self.clone()))
    }
}

impl TraversalModel for ElevationTraversalModel {
    fn input_features(&self) -> Vec<String> {
        vec![
            String::from(fieldname::EDGE_DISTANCE),
            String::from(fieldname::EDGE_GRADE),
        ]
    }

    fn output_features(&self) -> Vec<(String, StateFeature)> {
        vec![
            (
                String::from(fieldname::TRIP_ELEVATION_GAIN),
                StateFeature::Distance {
                    value: Length::ZERO,
                    accumulator: true,
                },
            ),
            (
                String::from(fieldname::TRIP_ELEVATION_LOSS),
                StateFeature::Grade {
                    value: Ratio::ZERO,
                    accumulator: true,
                },
            ),
        ]
    }

    /// compute the change in elevation along this edge and store it to the state vector
    fn traverse_edge(
        &self,
        _trajectory: (&Vertex, &Edge, &Vertex),
        state: &mut Vec<StateVariable>,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        let distance = state_model.get_distance(state, fieldname::EDGE_DISTANCE)?;
        let grade = state_model.get_grade(state, fieldname::EDGE_GRADE)?;
        let elevation_change = ElevationChange::new(distance, grade).map_err(|e| {
            TraversalModelError::TraversalModelFailure(format!("Elevation change error: {}", e))
        })?;
        elevation_change.add_elevation_to_state(state, state_model)?;
        Ok(())
    }

    /// we do not currently have the data available to estimate elevation changes using only pairs of coordinates
    fn estimate_traversal(
        &self,
        _od: (&Vertex, &Vertex),
        _state: &mut Vec<StateVariable>,
        _state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        Ok(())
    }
}
