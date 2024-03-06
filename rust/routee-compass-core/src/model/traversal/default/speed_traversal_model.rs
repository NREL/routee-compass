use crate::model::road_network::edge_id::EdgeId;
use crate::model::state::state_model::StateModel;
use crate::model::traversal::traversal_model::TraversalModel;
use crate::model::unit::as_f64::AsF64;
use crate::model::unit::{Distance, Time, BASE_DISTANCE_UNIT};
use crate::model::{
    property::{edge::Edge, vertex::Vertex},
    traversal::{
        state::{state_variable::StateVar, traversal_state::TraversalState},
        traversal_model_error::TraversalModelError,
    },
    unit::Speed,
};
use crate::util::geo::haversine;
use std::sync::Arc;

use super::speed_traversal_engine::SpeedTraversalEngine;

pub struct SpeedTraversalModel {
    engine: Arc<SpeedTraversalEngine>,
    state_model: Arc<StateModel>,
}

impl SpeedTraversalModel {
    pub fn new(
        engine: Arc<SpeedTraversalEngine>,
        state_model: Arc<StateModel>,
    ) -> SpeedTraversalModel {
        SpeedTraversalModel {
            engine,
            state_model,
        }
    }
}

impl TraversalModel for SpeedTraversalModel {
    // fn state_variable_names(&self) -> Vec<String> {
    //     vec![String::from("distance"), String::from("time")]
    // }

    // fn initial_state(&self) -> TraversalState {
    //     vec![StateVar(0.0), StateVar(0.0)]
    // }

    // fn get_state_variable(
    //     &self,
    //     key: &str,
    //     state: &[StateVar],
    // ) -> Result<StateVar, TraversalModelError> {
    //     let index = match key {
    //         "distance" => Ok(0),
    //         "time" => Ok(1),
    //         _ => Err(TraversalModelError::InternalError(format!(
    //             "unknown state variable {}, should be one of [distance, time]",
    //             key
    //         ))),
    //     }?;
    //     let value_f64 = state.get(index).ok_or_else(|| {
    //         TraversalModelError::InternalError(format!(
    //             "state variable {} with index {} not found in state",
    //             key, index
    //         ))
    //     })?;
    //     Ok(*value_f64)
    // }

    // fn serialize_state(&self, state: &[StateVar]) -> serde_json::Value {
    //     let distance = get_distance_from_state(state);
    //     let time = get_time_from_state(state);
    //     serde_json::json!({
    //         "distance": distance,
    //         "time": time,
    //     })
    // }

    // fn serialize_state_info(&self, _state: &[StateVar]) -> serde_json::Value {
    //     serde_json::json!({
    //         "distance_unit": self.engine.distance_unit,
    //         "time_unit": self.engine.time_unit,
    //     })
    // }

    fn traverse_edge(
        &self,
        _src: &Vertex,
        edge: &Edge,
        _dst: &Vertex,
        state: &[StateVar],
    ) -> Result<TraversalState, TraversalModelError> {
        let mut updated = state.to_vec();
        let distance = BASE_DISTANCE_UNIT.convert(edge.distance, self.engine.distance_unit);
        let speed = get_speed(&self.engine.speed_table, edge.edge_id)?;
        let time = Time::create(
            speed,
            self.engine.speed_unit,
            distance,
            self.engine.distance_unit,
            self.engine.time_unit,
        )?;
        self.state_model
            .update_add(&mut updated, "time", &StateVar(time.as_f64()))?;
        Ok(updated)
    }

    fn access_edge(
        &self,
        _v1: &Vertex,
        _src: &Edge,
        _v2: &Vertex,
        _dst: &Edge,
        _v3: &Vertex,
        _state: &[StateVar],
    ) -> Result<Option<TraversalState>, TraversalModelError> {
        Ok(None)
    }

    fn estimate_traversal(
        &self,
        src: &Vertex,
        dst: &Vertex,
        state: &[StateVar],
    ) -> Result<TraversalState, TraversalModelError> {
        let mut updated = state.to_vec();
        let distance =
            haversine::coord_distance(&src.coordinate, &dst.coordinate, self.engine.distance_unit)
                .map_err(TraversalModelError::NumericError)?;

        if distance == Distance::ZERO {
            return Ok(state.to_vec());
        }

        let time = Time::create(
            self.engine.max_speed,
            self.engine.speed_unit,
            distance,
            self.engine.distance_unit,
            self.engine.time_unit,
        )?;
        self.state_model
            .update_add(&mut updated, "time", &StateVar(time.as_f64()))?;
        // let updated_state = update_state(state, distance, time);
        Ok(updated)
    }
}

/// look up a speed from the speed table
pub fn get_speed(speed_table: &[Speed], edge_id: EdgeId) -> Result<Speed, TraversalModelError> {
    let speed: &Speed = speed_table.get(edge_id.as_usize()).ok_or_else(|| {
        TraversalModelError::MissingIdInTabularCostFunction(
            format!("{}", edge_id),
            String::from("EdgeId"),
            String::from("speed table"),
        )
    })?;
    Ok(*speed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::unit::Distance;
    use crate::model::{
        property::{edge::Edge, vertex::Vertex},
        road_network::{edge_id::EdgeId, vertex_id::VertexId},
    };
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
        let state_model = StateModel::new(json!({
            "distance": { "type": "distance", "unit": "kilometers"},
            "time": { "type": "time", "unit": "seconds"}
        }))
        .unwrap();
        let model = SpeedTraversalModel::new(Arc::new(engine), Arc::new(state_model));
        let initial = model.initial_state();
        let v = mock_vertex();
        let e1 = mock_edge(0);
        // 100 meters @ 10kph should take 36 seconds ((0.1/10) * 3600)
        let result = model.traverse_edge(&v, &e1, &v, &initial).unwrap();
        let expected = 36.0;
        // approx_eq(result.total_cost.into(), expected, 0.001);
        // approx_eq(result.updated_state[1].into(), expected, 0.001);
        approx_eq(result[1].into(), expected, 0.001);
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
        let state_model = StateModel::new(json!({
            "distance": { "type": "distance", "unit": "kilometers"},
            "time": { "type": "time", "unit": "seconds"}
        }))
        .unwrap();
        let model = SpeedTraversalModel::new(Arc::new(engine), Arc::new(state_model));
        let initial = model.initial_state();
        let v = mock_vertex();
        let e1 = mock_edge(0);
        // 100 meters @ 10kph should take 36,000 milliseconds ((0.1/10) * 3600000)
        let result = model.traverse_edge(&v, &e1, &v, &initial).unwrap();
        let expected = 36000.0;
        // approx_eq(result.total_cost.into(), expected, 0.001);
        // approx_eq(result.updated_state[1].into(), expected, 0.001);
        approx_eq(result[1].into(), expected, 0.001);
    }
}
