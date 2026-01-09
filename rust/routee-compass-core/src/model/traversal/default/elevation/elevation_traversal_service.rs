use std::sync::Arc;

use uom::{si::f64::Length, ConstZero};

use crate::model::{
    state::{InputFeature, StateModel, StateVariableConfig},
    traversal::{
        default::{elevation::ElevationTraversalModel, fieldname},
        TraversalModel, TraversalModelError, TraversalModelService,
    },
    unit::DistanceUnit,
};

#[derive(Clone, Debug)]
pub struct ElevationTraversalService {}

impl TraversalModelService for ElevationTraversalService {
    fn input_features(&self) -> Vec<InputFeature> {
        vec![
            InputFeature::Distance {
                name: String::from(fieldname::EDGE_DISTANCE),
                unit: None,
            },
            InputFeature::Ratio {
                name: String::from(fieldname::EDGE_GRADE),
                unit: None,
            },
        ]
    }

    fn output_features(&self) -> Vec<(String, StateVariableConfig)> {
        vec![
            (
                String::from(fieldname::TRIP_ELEVATION_GAIN),
                StateVariableConfig::Distance {
                    initial: Length::ZERO,
                    accumulator: true,
                    output_unit: Some(DistanceUnit::default()),
                },
            ),
            (
                String::from(fieldname::TRIP_ELEVATION_LOSS),
                StateVariableConfig::Distance {
                    initial: Length::ZERO,
                    accumulator: true,
                    output_unit: Some(DistanceUnit::default()),
                },
            ),
        ]
    }

    fn build(
        &self,
        _query: &serde_json::Value,
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
        let edge_grade_idx = state_model.get_index(fieldname::EDGE_GRADE).map_err(|e| {
            TraversalModelError::BuildError(format!("Failed to find EDGE_GRADE index: {}", e))
        })?;
        let trip_elevation_gain_idx = state_model
            .get_index(fieldname::TRIP_ELEVATION_GAIN)
            .map_err(|e| {
                TraversalModelError::BuildError(format!(
                    "Failed to find TRIP_ELEVATION_GAIN index: {}",
                    e
                ))
            })?;
        let trip_elevation_loss_idx = state_model
            .get_index(fieldname::TRIP_ELEVATION_LOSS)
            .map_err(|e| {
                TraversalModelError::BuildError(format!(
                    "Failed to find TRIP_ELEVATION_LOSS index: {}",
                    e
                ))
            })?;
        let traversal_model = ElevationTraversalModel {
            edge_distance_idx,
            edge_grade_idx,
            trip_elevation_gain_idx,
            trip_elevation_loss_idx,
        };
        Ok(Arc::new(traversal_model))
    }
}
