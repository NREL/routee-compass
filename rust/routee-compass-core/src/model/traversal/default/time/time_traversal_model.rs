use super::time_configuration::TimeConfiguration;
use crate::{
    model::{
        network::{Edge, Vertex},
        state::{InputFeature, OutputFeature, StateModel, StateVariable},
        traversal::{
            default::fieldname, TraversalModel, TraversalModelError, TraversalModelService,
        },
        unit::{baseunit, Convert, Distance, DistanceUnit, Time, TimeUnit},
    },
    util::geo::haversine,
};
use std::{borrow::Cow, sync::Arc};

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
                String::from(fieldname::EDGE_DISTANCE),
                InputFeature::Distance(baseunit::DISTANCE_UNIT),
            ),
            (
                String::from(fieldname::EDGE_SPEED),
                InputFeature::Speed(baseunit::SPEED_UNIT),
            ),
        ]
    }

    fn output_features(&self) -> Vec<(String, OutputFeature)> {
        vec![
            (
                String::from(fieldname::EDGE_TIME),
                OutputFeature::Time {
                    time_unit: self.time_unit,
                    initial: Time::ZERO,
                    accumulator: false,
                },
            ),
            (
                String::from(fieldname::TRIP_TIME),
                OutputFeature::Time {
                    time_unit: self.time_unit,
                    initial: Time::ZERO,
                    accumulator: true,
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
            state_model.get_distance(state, fieldname::EDGE_DISTANCE, None)?;
        let (speed, speed_unit) = state_model.get_speed(state, fieldname::EDGE_SPEED, None)?;

        let (t, tu) = Time::create((&distance, distance_unit), (&speed, speed_unit))?;
        let mut edge_time = Cow::Owned(t);
        tu.convert(&mut edge_time, &self.time_unit)?;

        state_model.add_time(state, fieldname::TRIP_TIME, &edge_time, &self.time_unit)?;
        state_model.set_time(state, fieldname::EDGE_TIME, &edge_time, &self.time_unit)?;

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
        let distance_unit = DistanceUnit::Meters;
        let (speed, speed_unit) = state_model.get_speed(state, fieldname::EDGE_SPEED, None)?;

        let (t, tu) = Time::create((&distance, &distance_unit), (&speed, speed_unit))?;
        let mut edge_time = Cow::Owned(t);
        tu.convert(&mut edge_time, &self.time_unit)?;

        state_model.add_time(state, fieldname::TRIP_TIME, &edge_time, &self.time_unit)?;
        state_model.set_time(state, fieldname::EDGE_TIME, &edge_time, &self.time_unit)?;

        Ok(())
    }
}
