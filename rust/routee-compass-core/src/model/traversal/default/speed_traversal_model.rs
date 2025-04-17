use super::speed_traversal_engine::SpeedTraversalEngine;
use crate::model::network::edge_id::EdgeId;
use crate::model::network::{Edge, Vertex};
use crate::model::state::StateFeature;
use crate::model::state::StateModel;
use crate::model::state::StateVariable;
use crate::model::traversal::traversal_model::TraversalModel;
use crate::model::unit::{baseunit, Convert, Distance, SpeedUnit, Time};
use crate::model::{traversal::traversal_model_error::TraversalModelError, unit::Speed};
use crate::util::geo::haversine;
use std::borrow::Cow;
use std::sync::Arc;

pub struct SpeedTraversalModel {
    engine: Arc<SpeedTraversalEngine>,
    speed_limit: Option<(Speed, SpeedUnit)>,
}

impl SpeedTraversalModel {
    pub fn new(
        engine: Arc<SpeedTraversalEngine>,
        speed_limit: Option<(Speed, SpeedUnit)>,
    ) -> Result<SpeedTraversalModel, TraversalModelError> {
        if let Some((max_speed, max_speed_unit)) = speed_limit {
            let mut max_speed_convert = Cow::Owned(max_speed);
            max_speed_unit.convert(&mut max_speed_convert, &engine.speed_unit)?;
            let converted_speed_unit = engine.speed_unit;
            Ok(SpeedTraversalModel {
                engine,
                speed_limit: Some((max_speed_convert.into_owned(), converted_speed_unit)),
            })
        } else {
            Ok(SpeedTraversalModel {
                engine,
                speed_limit: None,
            })
        }
    }
    const DISTANCE: &'static str = "distance";
    const TIME: &'static str = "time";
}

impl TraversalModel for SpeedTraversalModel {
    fn traverse_edge(
        &self,
        trajectory: (&Vertex, &Edge, &Vertex),
        state: &mut Vec<StateVariable>,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        let (_, edge, _) = trajectory;
        let mut distance = Cow::Borrowed(&edge.distance);
        baseunit::DISTANCE_UNIT.convert(&mut distance, &self.engine.distance_unit)?;
        let dist_converted = distance.into_owned();

        let lookup_speed = get_speed(&self.engine.speed_table, edge.edge_id)?;
        let speed = match self.speed_limit {
            // speed unit here is unused since we've already converted into the same unit as the speed model
            Some((speed_limit, _speed_unit)) => {
                if lookup_speed > speed_limit {
                    speed_limit
                } else {
                    lookup_speed
                }
            }
            None => lookup_speed,
        };

        let (t, tu) = Time::create(
            (&dist_converted, &self.engine.distance_unit),
            (&speed, &self.engine.speed_unit),
        )?;
        let mut edge_time = Cow::Owned(t);
        tu.convert(&mut edge_time, &self.engine.time_unit)?;

        state_model.add_time(
            state,
            &Self::TIME.into(),
            &edge_time,
            &self.engine.time_unit,
        )?;
        state_model.add_distance(
            state,
            &Self::DISTANCE.into(),
            &dist_converted,
            &self.engine.distance_unit,
        )?;
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
            haversine::coord_distance(&src.coordinate, &dst.coordinate, self.engine.distance_unit)
                .map_err(|e| {
                    TraversalModelError::TraversalModelFailure(format!(
                        "could not compute haversine distance between {} and {}: {}",
                        src, dst, e
                    ))
                })?;

        if distance == Distance::ZERO {
            return Ok(());
        }

        let max_speed = match self.speed_limit {
            Some((speed_limit, _speed_unit)) => speed_limit,
            None => self.engine.max_speed,
        };

        let (t, tu) = Time::create(
            (&distance, &self.engine.distance_unit),
            (&max_speed, &self.engine.speed_unit),
        )?;
        let mut edge_time = Cow::Owned(t);
        tu.convert(&mut edge_time, &self.engine.time_unit)?;

        state_model.add_time(
            state,
            &Self::TIME.into(),
            &edge_time,
            &self.engine.time_unit,
        )?;
        state_model.add_distance(
            state,
            &Self::DISTANCE.into(),
            &distance,
            &self.engine.distance_unit,
        )?;

        Ok(())
    }
    /// track the time state feature
    fn state_features(&self) -> Vec<(String, StateFeature)> {
        vec![
            (
                String::from(Self::TIME),
                StateFeature::Time {
                    time_unit: self.engine.time_unit,
                    initial: Time::ZERO,
                },
            ),
            (
                String::from(Self::DISTANCE),
                StateFeature::Distance {
                    distance_unit: self.engine.distance_unit,
                    initial: Distance::ZERO,
                },
            ),
        ]
    }
}

/// look up a speed from the speed table
pub fn get_speed(speed_table: &[Speed], edge_id: EdgeId) -> Result<Speed, TraversalModelError> {
    let speed: &Speed = speed_table.get(edge_id.as_usize()).ok_or_else(|| {
        TraversalModelError::TraversalModelFailure(format!(
            "could not find expected index {} in speed table",
            edge_id
        ))
    })?;
    Ok(*speed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::network::{Edge, EdgeId, Vertex, VertexId};
    use crate::model::unit::{Distance, DistanceUnit, SpeedUnit, TimeUnit};
    use crate::util::geo::InternalCoord;
    use geo::coord;
    use std::path::PathBuf;

    fn mock_vertex() -> Vertex {
        Vertex {
            vertex_id: VertexId(0),
            coordinate: InternalCoord(coord! {x: -86.67, y: 36.12}),
        }
    }
    fn mock_edge(edge_id: usize) -> Edge {
        Edge {
            edge_id: EdgeId(edge_id),
            src_vertex_id: VertexId(0),
            dst_vertex_id: VertexId(1),
            distance: Distance::from(100.0),
        }
    }
    fn filepath() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("model")
            .join("traversal")
            .join("default")
            .join("test")
            .join("velocities.txt")
    }

    fn approx_eq(a: f64, b: f64, error: f64) {
        let result = match (a, b) {
            (c, d) if c < d => d - c < error,
            (c, d) if c > d => c - d < error,
            (_, _) => true,
        };
        assert!(
            result,
            "{} ~= {} is not true within an error of {}",
            a, b, error
        )
    }

    #[test]
    fn test_edge_cost_lookup_with_seconds_time_unit() {
        let file = filepath();
        let engine =
            SpeedTraversalEngine::new(&file, SpeedUnit::KPH, None, Some(TimeUnit::Seconds))
                .unwrap();
        let state_model = Arc::new(
            StateModel::empty()
                .extend(vec![
                    (
                        String::from("distance"),
                        StateFeature::Distance {
                            distance_unit: DistanceUnit::Kilometers,
                            initial: Distance::from(0.0),
                        },
                    ),
                    (
                        String::from("time"),
                        StateFeature::Time {
                            time_unit: TimeUnit::Seconds,
                            initial: Time::from(0.0),
                        },
                    ),
                ])
                .unwrap(),
        );
        let model: SpeedTraversalModel = SpeedTraversalModel::new(Arc::new(engine), None).unwrap();
        let mut state = state_model.initial_state().unwrap();
        let v = mock_vertex();
        let e1 = mock_edge(0);
        // 100 meters @ 10kph should take 36 seconds ((0.1/10) * 3600)
        model
            .traverse_edge((&v, &e1, &v), &mut state, &state_model)
            .unwrap();

        let expected = 36.0;
        approx_eq(state[1].into(), expected, 0.001);
    }

    #[test]
    fn test_edge_cost_lookup_with_milliseconds_time_unit() {
        let file = filepath();
        let engine =
            SpeedTraversalEngine::new(&file, SpeedUnit::KPH, None, Some(TimeUnit::Milliseconds))
                .unwrap();
        let state_model = Arc::new(
            StateModel::empty()
                .extend(vec![
                    (
                        String::from("distance"),
                        StateFeature::Distance {
                            distance_unit: DistanceUnit::Kilometers,
                            initial: Distance::from(0.0),
                        },
                    ),
                    (
                        String::from("time"),
                        StateFeature::Time {
                            time_unit: TimeUnit::Milliseconds,
                            initial: Time::from(0.0),
                        },
                    ),
                ])
                .unwrap(),
        );
        let model = SpeedTraversalModel::new(Arc::new(engine), None);
        let mut state = state_model.initial_state().unwrap();
        let v = mock_vertex();
        let e1 = mock_edge(0);
        // 100 meters @ 10kph should take 36,000 milliseconds ((0.1/10) * 3600000)
        model
            .unwrap()
            .traverse_edge((&v, &e1, &v), &mut state, &state_model)
            .unwrap();
        let expected = 36000.0;
        approx_eq(state[1].into(), expected, 0.001);
    }

    #[test]
    fn test_speed_limit_enforcement() {
        let file = filepath();
        let engine = Arc::new(
            SpeedTraversalEngine::new(&file, SpeedUnit::KPH, None, Some(TimeUnit::Seconds))
                .unwrap(),
        );

        // We know from the test data that edge 0 has a speed of 10 kph, so set a limit of 5 kph
        let speed_limit = Some((Speed::from(5.0), SpeedUnit::KPH));

        let state_model = Arc::new(
            StateModel::empty()
                .extend(vec![
                    (
                        String::from("distance"),
                        StateFeature::Distance {
                            distance_unit: DistanceUnit::Kilometers,
                            initial: Distance::from(0.0),
                        },
                    ),
                    (
                        String::from("time"),
                        StateFeature::Time {
                            time_unit: TimeUnit::Seconds,
                            initial: Time::from(0.0),
                        },
                    ),
                ])
                .unwrap(),
        );

        // Create model with speed limit
        let model_with_limit = SpeedTraversalModel::new(engine.clone(), speed_limit);
        // Create model without speed limit for comparison
        let model_without_limit = SpeedTraversalModel::new(engine, None);

        let mut state_with_limit = state_model.initial_state().unwrap();
        let mut state_without_limit = state_model.initial_state().unwrap();

        let v = mock_vertex();
        let e = mock_edge(0);

        // Traverse with speed limit
        model_with_limit
            .unwrap()
            .traverse_edge((&v, &e, &v), &mut state_with_limit, &state_model)
            .unwrap();

        // Traverse without speed limit
        model_without_limit
            .unwrap()
            .traverse_edge((&v, &e, &v), &mut state_without_limit, &state_model)
            .unwrap();

        // The time with speed limit should be about twice the time without limit
        // because we set the limit to half the edge speed (5 kph vs 10 kph)
        let time_with_limit: f64 = state_with_limit[1].into();
        let time_without_limit: f64 = state_without_limit[1].into();

        // 100 meters @ 5kph should take 72 seconds ((0.1/5) * 3600)
        let expected_time_with_limit = 72.0;
        // 100 meters @ 10kph should take 36 seconds ((0.1/10) * 3600)
        let expected_time_without_limit = 36.0;

        approx_eq(time_with_limit, expected_time_with_limit, 0.001);
        approx_eq(time_without_limit, expected_time_without_limit, 0.001);

        // Verify that time with limit is about double the time without limit
        approx_eq(time_with_limit / time_without_limit, 2.0, 0.001);
    }
    #[test]
    fn test_speed_limit_unit_conversion() {
        let file = filepath();

        // Create engine with kilometers per hour as its speed unit
        let engine = Arc::new(
            SpeedTraversalEngine::new(&file, SpeedUnit::KPH, None, Some(TimeUnit::Seconds))
                .unwrap(),
        );

        // Set speed limit in miles per hour (5 mph ≈ 8 kph)
        let speed_limit_mph = Some((Speed::from(5.0), SpeedUnit::MPH));

        // Create a model with the speed limit in mph
        let model_mph_limit = SpeedTraversalModel::new(engine.clone(), speed_limit_mph);

        // For comparison, create a model with equivalent speed limit directly in kph
        let speed_limit_kph = Some((Speed::from(8.04672), SpeedUnit::KPH));
        let model_kph_limit = SpeedTraversalModel::new(engine, speed_limit_kph);

        let state_model = Arc::new(
            StateModel::empty()
                .extend(vec![
                    (
                        String::from("distance"),
                        StateFeature::Distance {
                            distance_unit: DistanceUnit::Kilometers,
                            initial: Distance::from(0.0),
                        },
                    ),
                    (
                        String::from("time"),
                        StateFeature::Time {
                            time_unit: TimeUnit::Seconds,
                            initial: Time::from(0.0),
                        },
                    ),
                ])
                .unwrap(),
        );

        let mut state_mph = state_model.initial_state().unwrap();
        let mut state_kph = state_model.initial_state().unwrap();

        let v = mock_vertex();
        let e = mock_edge(0); // this edge has a speed of 10 kph in test data

        // Traverse with mph-based limit
        model_mph_limit
            .unwrap()
            .traverse_edge((&v, &e, &v), &mut state_mph, &state_model)
            .unwrap();

        // Traverse with kph-based limit
        model_kph_limit
            .unwrap()
            .traverse_edge((&v, &e, &v), &mut state_kph, &state_model)
            .unwrap();

        // Both should produce virtually identical traversal times since the speed limits
        // should be equivalent after unit conversion
        let time_mph: f64 = state_mph[1].into();
        let time_kph: f64 = state_kph[1].into();

        // Verify times are nearly identical
        approx_eq(time_mph, time_kph, 0.1);
    }
}
