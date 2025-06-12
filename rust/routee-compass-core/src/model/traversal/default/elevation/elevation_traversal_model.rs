use std::sync::Arc;

use crate::model::{
    network::{Edge, Vertex},
    state::{InputFeature, OutputFeature, StateModel, StateVariable},
    traversal::{default::fieldname, TraversalModel, TraversalModelError, TraversalModelService},
    unit::{Distance, DistanceUnit},
};

use super::{ElevationChange, ElevationConfiguration};

#[derive(Clone, Debug)]
pub struct ElevationTraversalModel {
    distance_unit: DistanceUnit,
}

impl ElevationTraversalModel {
    pub fn new(config: &ElevationConfiguration) -> ElevationTraversalModel {
        ElevationTraversalModel {
            distance_unit: config.distance_unit,
        }
    }
}

impl TraversalModelService for ElevationTraversalModel {
    fn build(
        &self,
        _query: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModel>, TraversalModelError> {
        Ok(Arc::new(self.clone()))
    }
}

impl TraversalModel for ElevationTraversalModel {
    fn input_features(&self) -> Vec<(String, InputFeature)> {
        vec![
            (
                String::from(fieldname::EDGE_DISTANCE),
                InputFeature::Distance(None),
            ),
            (
                String::from(fieldname::EDGE_GRADE),
                InputFeature::Grade(Some(super::ELEVATION_GRADE_UNIT)),
            ),
        ]
    }

    fn output_features(&self) -> Vec<(String, OutputFeature)> {
        vec![
            (
                String::from(fieldname::TRIP_ELEVATION_GAIN),
                OutputFeature::Distance {
                    distance_unit: self.distance_unit,
                    initial: Distance::ZERO,
                    accumulator: true,
                },
            ),
            (
                String::from(fieldname::TRIP_ELEVATION_LOSS),
                OutputFeature::Distance {
                    distance_unit: self.distance_unit,
                    initial: Distance::ZERO,
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
        let (distance, distance_unit) =
            state_model.get_distance(state, fieldname::EDGE_DISTANCE, Some(&self.distance_unit))?;
        let (grade, grade_unit) = state_model.get_grade(
            state,
            fieldname::EDGE_GRADE,
            Some(&super::ELEVATION_GRADE_UNIT),
        )?;
        let elevation_change =
            ElevationChange::new((&distance, distance_unit), (&grade, grade_unit))?;
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
