use super::speed_traversal_engine::SpeedTraversalEngine;
use crate::model::network::edge_id::EdgeId;
use crate::model::network::{Edge, Vertex};
use crate::model::state::StateFeature;
use crate::model::state::StateModel;
use crate::model::state::StateVariable;
use crate::model::traversal::traversal_model::TraversalModel;
use crate::model::unit::{Distance, SpeedUnit, Time, BASE_DISTANCE_UNIT};
use crate::model::{traversal::traversal_model_error::TraversalModelError, unit::Speed};
use crate::util::geo::haversine;
use std::sync::Arc;

pub struct SpeedTraversalModel {
    engine: Arc<SpeedTraversalEngine>,

    max_speed: Option<Speed>, // max speed is converted to the speed unit of the engine
}

impl SpeedTraversalModel {
    pub fn new(
        engine: Arc<SpeedTraversalEngine>,
        max_speed: Option<(Speed, SpeedUnit)>,
    ) -> SpeedTraversalModel {
        if let Some((max_speed, max_speed_unit)) = max_speed {
            let converted_speed = max_speed_unit.convert(&max_speed, &engine.speed_unit);
            return SpeedTraversalModel {
                engine,
                max_speed: Some(converted_speed),
            };
        }

        SpeedTraversalModel {
            engine,
            max_speed: None,
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
        let distance = BASE_DISTANCE_UNIT.convert(&edge.distance, &self.engine.distance_unit);
        let speed = get_speed(&self.engine.speed_table, edge.edge_id)?;

        let speed = match self.max_speed {
            Some(max_speed) => {
                if speed > max_speed {
                    max_speed
                } else {
                    speed
                }
            }
            None => speed,
        };

        let edge_time = Time::create(
            &speed,
            &self.engine.speed_unit,
            &distance,
            &self.engine.distance_unit,
            &self.engine.time_unit,
        )?;

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

        let estimated_time = Time::create(
            &self.engine.max_speed,
            &self.engine.speed_unit,
            &distance,
            &self.engine.distance_unit,
            &self.engine.time_unit,
        )?;
        state_model.add_time(
            state,
            &Self::TIME.into(),
            &estimated_time,
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
    use crate::util::geo::coord::InternalCoord;
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
            distance: Distance::new(100.0),
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
        let engine = SpeedTraversalEngine::new(
            &file,
            SpeedUnit::KilometersPerHour,
            None,
            Some(TimeUnit::Seconds),
        )
        .unwrap();
        let state_model = Arc::new(
            StateModel::empty()
                .extend(vec![
                    (
                        String::from("distance"),
                        StateFeature::Distance {
                            distance_unit: DistanceUnit::Kilometers,
                            initial: Distance::new(0.0),
                        },
                    ),
                    (
                        String::from("time"),
                        StateFeature::Time {
                            time_unit: TimeUnit::Seconds,
                            initial: Time::new(0.0),
                        },
                    ),
                ])
                .unwrap(),
        );
        let model: SpeedTraversalModel = SpeedTraversalModel::new(Arc::new(engine), None);
        let mut state = state_model.initial_state().unwrap();
        let v = mock_vertex();
        let e1 = mock_edge(0);
        // 100 meters @ 10kph should take 36 seconds ((0.1/10) * 3600)
        model
            .traverse_edge((&v, &e1, &v), &mut state, &state_model)
            .unwrap();

        let expected = 36.0;
        // approx_eq(result.total_cost.into(), expected, 0.001);
        // approx_eq(result.updated_state[1].into(), expected, 0.001);
        approx_eq(state[1].into(), expected, 0.001);
    }

    #[test]
    fn test_edge_cost_lookup_with_milliseconds_time_unit() {
        let file = filepath();
        let engine = SpeedTraversalEngine::new(
            &file,
            SpeedUnit::KilometersPerHour,
            None,
            Some(TimeUnit::Milliseconds),
        )
        .unwrap();
        let state_model = Arc::new(
            StateModel::empty()
                .extend(vec![
                    (
                        String::from("distance"),
                        StateFeature::Distance {
                            distance_unit: DistanceUnit::Kilometers,
                            initial: Distance::new(0.0),
                        },
                    ),
                    (
                        String::from("time"),
                        StateFeature::Time {
                            time_unit: TimeUnit::Milliseconds,
                            initial: Time::new(0.0),
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
            .traverse_edge((&v, &e1, &v), &mut state, &state_model)
            .unwrap();
        let expected = 36000.0;
        // approx_eq(result.total_cost.into(), expected, 0.001);
        // approx_eq(result.updated_state[1].into(), expected, 0.001);
        approx_eq(state[1].into(), expected, 0.001);
    }
}
