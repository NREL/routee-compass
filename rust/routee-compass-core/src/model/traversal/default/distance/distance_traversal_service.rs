use super::DistanceTraversalModel;
use crate::model::state::{InputFeature, StateModel, StateVariableConfig};
use crate::model::traversal::default::fieldname;
use crate::model::traversal::traversal_model::TraversalModel;
use crate::model::traversal::TraversalModelError;
use crate::model::traversal::TraversalModelService;
use crate::model::unit::DistanceUnit;
use std::sync::Arc;
use uom::si::f64::Length;
use uom::ConstZero;

pub struct DistanceTraversalService {
    pub distance_unit: DistanceUnit,
    pub include_trip_distance: bool,
}

impl DistanceTraversalService {
    pub fn new(
        distance_unit: DistanceUnit,
        include_trip_distance: bool,
    ) -> DistanceTraversalService {
        Self {
            distance_unit,
            include_trip_distance,
        }
    }
}

impl TraversalModelService for DistanceTraversalService {
    fn input_features(&self) -> Vec<InputFeature> {
        vec![]
    }

    fn output_features(&self) -> Vec<(String, StateVariableConfig)> {
        let mut features = vec![(
            String::from(fieldname::EDGE_DISTANCE),
            StateVariableConfig::Distance {
                initial: Length::ZERO,
                accumulator: false,
                output_unit: Some(self.distance_unit),
            },
        )];
        if self.include_trip_distance {
            features.push((
                String::from(fieldname::TRIP_DISTANCE),
                StateVariableConfig::Distance {
                    initial: Length::ZERO,
                    accumulator: true,
                    output_unit: Some(self.distance_unit),
                },
            ));
        }
        features
    }

    fn build(
        &self,
        _parameters: &serde_json::Value,
        state_model: Arc<StateModel>,
    ) -> Result<Arc<dyn TraversalModel>, TraversalModelError> {
        let edge_distance_idx = state_model
            .get_index(fieldname::EDGE_DISTANCE)
            .map_err(|e| {
                TraversalModelError::BuildError(format!(
                    "Failed to find EDGE_DISTANCE index: {}",
                    e
                ))
            })?;

        let trip_distance_idx = if self.include_trip_distance {
            Some(
                state_model
                    .get_index(fieldname::TRIP_DISTANCE)
                    .map_err(|e| {
                        TraversalModelError::BuildError(format!(
                            "Failed to find TRIP_DISTANCE index: {}",
                            e
                        ))
                    })?,
            )
        } else {
            None
        };

        let m: Arc<dyn TraversalModel> = Arc::new(DistanceTraversalModel::new(
            self.distance_unit,
            self.include_trip_distance,
            edge_distance_idx,
            trip_distance_idx,
        ));
        Ok(m)
    }
}
