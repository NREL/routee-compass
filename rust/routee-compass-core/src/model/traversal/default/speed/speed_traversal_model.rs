use super::speed_traversal_engine::SpeedTraversalEngine;
use crate::model::network::edge_id::EdgeId;
use crate::model::network::{Edge, Vertex};
use crate::model::state::StateModel;
use crate::model::state::StateVariable;
use crate::model::state::{InputFeature, OutputFeature};
use crate::model::traversal::traversal_model::TraversalModel;
use crate::model::unit::{Convert, SpeedUnit};
use crate::model::{traversal::traversal_model_error::TraversalModelError, unit::Speed};
use std::borrow::Cow;
use std::sync::Arc;

pub struct SpeedTraversalModel {
    engine: Arc<SpeedTraversalEngine>,
    speed_limit: Option<(Speed, SpeedUnit)>,
}

impl SpeedTraversalModel {
    const EDGE_DISTANCE: &'static str = "edge_distance";
    const EDGE_SPEED: &'static str = "edge_speed";

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
}

impl TraversalModel for SpeedTraversalModel {
    fn input_features(&self) -> Vec<(String, InputFeature)> {
        vec![(
            String::from(Self::EDGE_DISTANCE),
            InputFeature::Distance(None),
        )]
    }

    fn output_features(&self) -> Vec<(String, OutputFeature)> {
        vec![(
            String::from(Self::EDGE_SPEED),
            OutputFeature::Speed {
                speed_unit: self.engine.speed_unit,
                initial: Speed::ZERO,
            },
        )]
    }

    /// records the speed that will be driven over this edge into the state vector.
    fn traverse_edge(
        &self,
        trajectory: (&Vertex, &Edge, &Vertex),
        state: &mut Vec<StateVariable>,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        let (_, edge, _) = trajectory;
        let lookup_speed = get_speed(&self.engine.speed_table, edge.edge_id)?;
        let speed = apply_speed_limit(lookup_speed, self.speed_limit.as_ref());
        state_model.add_speed(state, super::EDGE_SPEED, &speed, &self.engine.speed_unit)?;
        Ok(())
    }

    /// (over-)estimates speed over remainder of the trip as the maximum-possible speed value.
    fn estimate_traversal(
        &self,
        _od: (&Vertex, &Vertex),
        state: &mut Vec<StateVariable>,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        let speed = match self.speed_limit {
            Some((speed_limit, _speed_unit)) => speed_limit,
            None => self.engine.max_speed,
        };
        state_model.add_speed(state, super::EDGE_SPEED, &speed, &self.engine.speed_unit)?;

        Ok(())
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

fn apply_speed_limit(lookup_speed: Speed, speed_limit: Option<&(Speed, SpeedUnit)>) -> Speed {
    match speed_limit {
        // speed unit here is unused since we've already converted into the same unit as the speed model
        Some((speed_limit, _speed_unit)) => {
            if &lookup_speed > speed_limit {
                *speed_limit
            } else {
                lookup_speed
            }
        }
        None => lookup_speed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::network::{Edge, EdgeId, Vertex, VertexId};
    use crate::model::unit::{Distance, SpeedUnit};
    use crate::test::mock::traversal_model::TestTraversalModel;
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
    fn test_speed_traversal() {
        let file: PathBuf = filepath();
        let engine =
            SpeedTraversalEngine::new(&file, SpeedUnit::KPH).expect("test invariant failed");
        let speed_model =
            SpeedTraversalModel::new(Arc::new(engine), None).expect("test invariant failed");
        let test_model =
            TestTraversalModel::new(Arc::new(speed_model)).expect("test invariant failed");
        let state_model = StateModel::empty()
            .register(test_model.input_features(), test_model.output_features())
            .expect("test invariant failed");

        let mut state = state_model.initial_state().unwrap();
        let v = mock_vertex();
        let e1 = mock_edge(0);
        test_model
            .traverse_edge((&v, &e1, &v), &mut state, &state_model)
            .unwrap();

        let expected_speed = Speed::from(10.0);
        let expected_unit = SpeedUnit::KPH;
        let (result_speed, result_unit) = state_model
            .get_speed(&state, "edge_speed", None)
            .expect("test invariant failed");
        assert_eq!(
            expected_speed, result_speed,
            "speed should match edge 0 in velocities.txt"
        );
        assert_eq!(
            expected_unit, *result_unit,
            "unit should match SpeedUnit of traversal model (KPH)"
        );
    }

    #[test]
    fn test_speed_limit_enforcement() {
        // We know from the test data that edge 0 has a speed of 10 kph, so set a limit of 5 kph
        let speed_limit_value = Speed::from(5.0);
        let speed_limit = Some((speed_limit_value, SpeedUnit::KPH));

        let file: PathBuf = filepath();
        let engine = Arc::new(
            SpeedTraversalEngine::new(&file, SpeedUnit::KPH).expect("test invariant failed"),
        );

        let regular_model =
            SpeedTraversalModel::new(engine.clone(), None).expect("test invariant failed");
        let limited_model =
            SpeedTraversalModel::new(engine.clone(), speed_limit).expect("test invariant failed");

        let test_regular_model =
            TestTraversalModel::new(Arc::new(regular_model)).expect("test invariant failed");
        let test_limited_model =
            TestTraversalModel::new(Arc::new(limited_model)).expect("test invariant failed");
        let state_model = StateModel::empty()
            .register(
                test_regular_model.input_features(),
                test_regular_model.output_features(),
            )
            .expect("test invariant failed");

        // // Create model with speed limit
        // let model_with_limit = SpeedTraversalModel::new(engine.clone(), speed_limit);
        // // Create model without speed limit for comparison
        // let model_without_limit = SpeedTraversalModel::new(engine, None);

        let mut state_with_limit = state_model.initial_state().unwrap();
        let mut state_without_limit = state_model.initial_state().unwrap();

        let v = mock_vertex();
        let e = mock_edge(0);

        // Traverse with speed limit
        test_limited_model
            .traverse_edge((&v, &e, &v), &mut state_with_limit, &state_model)
            .unwrap();

        // Traverse without speed limit
        test_regular_model
            .traverse_edge((&v, &e, &v), &mut state_without_limit, &state_model)
            .unwrap();

        // The time with speed limit should be about twice the time without limit
        // because we set the limit to half the edge speed (5 kph vs 10 kph)
        let (speed_with_limit, _) = state_model
            .get_speed(&state_with_limit, "edge_speed", None)
            .expect("test invariant failed");
        let (speed_without_limit, _) = state_model
            .get_speed(&state_without_limit, "edge_speed", None)
            .expect("test invariant failed");

        assert_eq!(
            speed_with_limit, speed_limit_value,
            "speed with limit should match the speed limit value"
        );
        assert_eq!(
            speed_without_limit,
            Speed::from(10.0),
            "speed without limit should match velocities.txt (10)"
        );

        // // 100 meters @ 5kph should take 72 seconds ((0.1/5) * 3600)
        // let expected_time_with_limit = 72.0;
        // // 100 meters @ 10kph should take 36 seconds ((0.1/10) * 3600)
        // let expected_time_without_limit = 36.0;

        // approx_eq(time_with_limit, expected_time_with_limit, 0.001);
        // approx_eq(time_without_limit, expected_time_without_limit, 0.001);

        // // Verify that time with limit is about double the time without limit
        // approx_eq(time_with_limit / time_without_limit, 2.0, 0.001);
    }
}
