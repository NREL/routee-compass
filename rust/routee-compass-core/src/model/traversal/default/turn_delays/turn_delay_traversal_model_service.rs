use super::TurnDelayTraversalModel;
use super::TurnDelayTraversalModelEngine;
use crate::model::state::{InputFeature, StateVariableConfig};
use crate::model::traversal::default::fieldname;
use crate::model::traversal::TraversalModel;
use crate::model::traversal::TraversalModelError;
use crate::model::traversal::TraversalModelService;
use crate::model::unit::TimeUnit;
use std::sync::Arc;
use uom::{si::f64::Time, ConstZero};

pub struct TurnDelayTraversalModelService {
    pub engine: Arc<TurnDelayTraversalModelEngine>,
    pub include_trip_time: bool,
}

impl TurnDelayTraversalModelService {}

impl TraversalModelService for TurnDelayTraversalModelService {
    fn input_features(&self) -> Vec<InputFeature> {
        vec![]
    }

    fn output_features(&self) -> Vec<(String, StateVariableConfig)> {
        let mut features = vec![
            (
                String::from(fieldname::EDGE_TURN_DELAY),
                StateVariableConfig::Time {
                    initial: Time::ZERO,
                    accumulator: false,
                    output_unit: Some(TimeUnit::Seconds),
                },
            ),
            (
                String::from(fieldname::EDGE_TIME),
                StateVariableConfig::Time {
                    initial: Time::ZERO,
                    accumulator: false,
                    output_unit: None,
                },
            ),
        ];
        if self.include_trip_time {
            features.push((
                String::from(fieldname::TRIP_TIME),
                StateVariableConfig::Time {
                    initial: Time::ZERO,
                    accumulator: true,
                    output_unit: None,
                },
            ));
        }
        features
    }

    fn build(
        &self,
        _query: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModel>, TraversalModelError> {
        let model = TurnDelayTraversalModel::new(self.engine.clone(), self.include_trip_time);
        Ok(Arc::new(model))
    }
}
