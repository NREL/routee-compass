use std::{borrow::Cow, sync::Arc};

use crate::{
    model::{
        network::{Edge, Vertex},
        state::{InputFeature, OutputFeature, StateModel, StateVariable},
        traversal::{TraversalModel, TraversalModelError, TraversalModelService},
        unit::{Convert, Distance, DistanceUnit, SpeedUnit, Time, TimeUnit},
    },
    util::geo::haversine,
};

use super::time_configuration::TimeConfiguration;

#[derive(Clone, Debug)]
pub struct TimeTraversalModel {
    time_unit: TimeUnit,
}

impl TimeTraversalModel {
    pub fn new(config: &TimeConfiguration) -> TimeTraversalModel {
        TimeTraversalModel {
            time_unit: config.time_unit,
        }
    }
}

impl TraversalModelService for TimeTraversalModel {
    fn build(
        &self,
        _query: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModel>, TraversalModelError> {
        Ok(Arc::new(self.clone()))
    }
}

impl TraversalModel for TimeTraversalModel {
    fn input_features(&self) -> Vec<(String, InputFeature)> {
        vec![
            (
                String::from(super::EDGE_DISTANCE),
                InputFeature::Distance(None),
            ),
            (String::from(super::EDGE_SPEED), InputFeature::Speed(None)),
        ]
    }

    fn output_features(&self) -> Vec<(String, OutputFeature)> {
        vec![
            (
                String::from(super::EDGE_TIME),
                OutputFeature::Time {
                    time_unit: self.time_unit,
                    initial: Time::ZERO,
                },
            ),
            (
                String::from(super::TRIP_TIME),
                OutputFeature::Time {
                    time_unit: self.time_unit,
                    initial: Time::ZERO,
                },
            ),
        ]
    }

    fn traverse_edge(
        &self,
        _trajectory: (&Vertex, &Edge, &Vertex),
        state: &mut Vec<StateVariable>,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        let (distance, distance_unit) =
            state_model.get_distance(state, super::EDGE_DISTANCE, None)?;
        let (speed, speed_unit) = state_model.get_speed(state, super::EDGE_SPEED, None)?;

        let (t, tu) = Time::create((&distance, distance_unit), (&speed, speed_unit))?;
        let mut edge_time = Cow::Owned(t);
        tu.convert(&mut edge_time, &self.time_unit)?;

        state_model.add_time(state, super::TRIP_TIME, &edge_time, &self.time_unit)?;
        state_model.set_time(state, super::EDGE_TIME, &edge_time, &self.time_unit)?;

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
            haversine::coord_distance_meters(&src.coordinate, &dst.coordinate).map_err(|e| {
                TraversalModelError::TraversalModelFailure(format!(
                    "could not compute haversine distance between {} and {}: {}",
                    src, dst, e
                ))
            })?;

        if distance == Distance::ZERO {
            return Ok(());
        }
        let (speed, speed_unit) = state_model.get_speed(state, super::EDGE_SPEED, None)?;

        let (t, tu) = Time::create((&distance, &DistanceUnit::Meters), (&speed, speed_unit))?;
        let mut edge_time = Cow::Owned(t);
        tu.convert(&mut edge_time, &self.time_unit)?;

        state_model.add_time(state, super::TRIP_TIME, &edge_time, &self.time_unit)?;
        state_model.set_time(state, super::EDGE_TIME, &edge_time, &self.time_unit)?;

        Ok(())
    }
}
