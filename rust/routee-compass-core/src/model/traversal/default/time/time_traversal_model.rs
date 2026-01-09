use uom::{
    si::f64::{Length, Time, Velocity},
    ConstZero,
};

use crate::{
    algorithm::search::SearchTree,
    model::{
        network::{Edge, Vertex},
        state::{InputFeature, StateModel, StateVariable, StateVariableConfig},
        traversal::{
            default::{fieldname, time::TimeTraversalConfig},
            TraversalModel, TraversalModelError, TraversalModelService,
        },
    },
    util::geo::haversine,
};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct TimeTraversalModel {
    config: TimeTraversalConfig,
    // Pre-resolved indices for performance
    edge_distance_idx: usize,
    edge_speed_idx: usize,
    edge_time_idx: usize,
    trip_time_idx: Option<usize>,
}

impl TimeTraversalModel {
    pub fn new(
        config: TimeTraversalConfig,
        edge_distance_idx: usize,
        edge_speed_idx: usize,
        edge_time_idx: usize,
        trip_time_idx: Option<usize>,
    ) -> TimeTraversalModel {
        TimeTraversalModel {
            config,
            edge_distance_idx,
            edge_speed_idx,
            edge_time_idx,
            trip_time_idx,
        }
    }
}

impl TraversalModelService for TimeTraversalModel {
    fn input_features(&self) -> Vec<InputFeature> {
        vec![
            InputFeature::Distance {
                name: String::from(fieldname::EDGE_DISTANCE),
                unit: None,
            },
            InputFeature::Speed {
                name: String::from(fieldname::EDGE_SPEED),
                unit: None,
            },
        ]
    }

    fn output_features(&self) -> Vec<(String, StateVariableConfig)> {
        let mut features = vec![(
            String::from(fieldname::EDGE_TIME),
            StateVariableConfig::Time {
                initial: Time::ZERO,
                accumulator: false,
                output_unit: Some(self.config.time_unit),
            },
        )];
        if self.config.include_trip_time.unwrap_or(true) {
            features.push((
                String::from(fieldname::TRIP_TIME),
                StateVariableConfig::Time {
                    initial: Time::ZERO,
                    accumulator: true,
                    output_unit: Some(self.config.time_unit),
                },
            ));
        }
        features
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

        let edge_speed_idx = state_model.get_index(fieldname::EDGE_SPEED).map_err(|e| {
            TraversalModelError::BuildError(format!("Failed to find EDGE_SPEED index: {}", e))
        })?;

        let edge_time_idx = state_model.get_index(fieldname::EDGE_TIME).map_err(|e| {
            TraversalModelError::BuildError(format!("Failed to find EDGE_TIME index: {}", e))
        })?;

        let trip_time_idx = if self.config.include_trip_time.unwrap_or(true) {
            Some(state_model.get_index(fieldname::TRIP_TIME).map_err(|e| {
                TraversalModelError::BuildError(format!("Failed to find TRIP_TIME index: {}", e))
            })?)
        } else {
            None
        };

        Ok(Arc::new(TimeTraversalModel::new(
            self.config.clone(),
            edge_distance_idx,
            edge_speed_idx,
            edge_time_idx,
            trip_time_idx,
        )))
    }
}

impl TraversalModel for TimeTraversalModel {
    fn name(&self) -> String {
        String::from("Time Traversal Model")
    }

    fn traverse_edge(
        &self,
        _trajectory: (&Vertex, &Edge, &Vertex),
        state: &mut Vec<StateVariable>,
        _tree: &SearchTree,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        let distance: Length = state_model.get_distance_by_index(state, self.edge_distance_idx)?;
        let speed: Velocity = state_model.get_speed_by_index(state, self.edge_speed_idx)?;

        let edge_time = distance / speed;

        if let Some(trip_time_idx) = self.trip_time_idx {
            state_model.add_time_by_index(state, trip_time_idx, &edge_time)?;
        }
        state_model.add_time_by_index(state, self.edge_time_idx, &edge_time)?;

        Ok(())
    }

    fn estimate_traversal(
        &self,
        od: (&Vertex, &Vertex),
        state: &mut Vec<StateVariable>,
        _tree: &SearchTree,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        let (src, dst) = od;
        let distance: Length = haversine::coord_distance(&src.coordinate, &dst.coordinate)
            .map_err(|e| {
                TraversalModelError::TraversalModelFailure(format!(
                    "could not compute haversine distance between {src} and {dst}: {e}"
                ))
            })?;

        if distance == Length::ZERO {
            return Ok(());
        }

        let speed = state_model.get_speed_by_index(state, self.edge_speed_idx)?;
        let time = distance / speed;

        if let Some(trip_time_idx) = self.trip_time_idx {
            state_model.add_time_by_index(state, trip_time_idx, &time)?;
        }
        state_model.add_time_by_index(state, self.edge_time_idx, &time)?;

        Ok(())
    }
}
