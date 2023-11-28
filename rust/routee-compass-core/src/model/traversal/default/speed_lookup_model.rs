use crate::model::road_network::edge_id::EdgeId;
use crate::util::fs::read_decoders;
use crate::util::geo::haversine;
use crate::util::unit::{Distance, DistanceUnit};
use crate::util::unit::{SpeedUnit, Time, TimeUnit, BASE_DISTANCE_UNIT, BASE_TIME_UNIT};
use crate::{
    model::{
        cost::Cost,
        property::{edge::Edge, vertex::Vertex},
        traversal::{
            state::{state_variable::StateVar, traversal_state::TraversalState},
            traversal_model::TraversalModel,
            traversal_model_error::TraversalModelError,
            traversal_result::TraversalResult,
        },
    },
    util::{fs::read_utils, unit::Speed},
};
use std::path::Path;

pub struct SpeedLookupModel {
    speed_table: Box<[Speed]>,
    speed_unit: SpeedUnit,
    output_time_unit: TimeUnit,
    output_distance_unit: DistanceUnit,
    max_speed: Speed,
}

impl SpeedLookupModel {
    pub fn new<P: AsRef<Path>>(
        speed_table_path: &P,
        speed_unit: SpeedUnit,
        output_distance_unit_opt: Option<DistanceUnit>,
        output_time_unit_opt: Option<TimeUnit>,
    ) -> Result<SpeedLookupModel, TraversalModelError> {
        let speed_table: Box<[Speed]> =
            read_utils::read_raw_file(speed_table_path, read_decoders::default, None).map_err(
                |e| {
                    TraversalModelError::FileReadError(
                        speed_table_path.as_ref().to_path_buf(),
                        e.to_string(),
                    )
                },
            )?;
        let max_speed = get_max_speed(&speed_table)?;
        let output_time_unit = output_time_unit_opt.unwrap_or(BASE_TIME_UNIT);
        let output_distance_unit = output_distance_unit_opt.unwrap_or(BASE_DISTANCE_UNIT);
        let model = SpeedLookupModel {
            speed_table,
            output_distance_unit,
            output_time_unit,
            speed_unit,
            max_speed,
        };
        Ok(model)
    }
}

impl TraversalModel for SpeedLookupModel {
    fn initial_state(&self) -> TraversalState {
        // distance, time
        vec![StateVar(0.0), StateVar(0.0)]
    }
    fn traversal_cost(
        &self,
        _src: &Vertex,
        edge: &Edge,
        _dst: &Vertex,
        state: &TraversalState,
    ) -> Result<TraversalResult, TraversalModelError> {
        let distance = BASE_DISTANCE_UNIT.convert(edge.distance, self.output_distance_unit);
        let speed = get_speed(&self.speed_table, edge.edge_id)?;
        let time = Time::create(
            speed,
            self.speed_unit,
            distance,
            self.output_distance_unit,
            self.output_time_unit.clone(),
        )?;

        let total_cost = Cost::from(time);
        let updated_state = update_state(state, distance, time);
        let result = TraversalResult {
            total_cost,
            updated_state,
        };
        Ok(result)
    }

    fn cost_estimate(
        &self,
        src: &Vertex,
        dst: &Vertex,
        _state: &TraversalState,
    ) -> Result<Cost, TraversalModelError> {
        let distance =
            haversine::coord_distance(src.coordinate, dst.coordinate, self.output_distance_unit)
                .map_err(TraversalModelError::NumericError)?;

        if distance == Distance::ZERO {
            return Ok(Cost::ZERO);
        }

        let time = Time::create(
            self.max_speed,
            self.speed_unit,
            distance,
            self.output_distance_unit,
            self.output_time_unit.clone(),
        )?;
        Ok(Cost::from(time))
    }

    fn serialize_state(&self, state: &TraversalState) -> serde_json::Value {
        let distance = get_distance_from_state(state);
        let time = get_time_from_state(state);
        serde_json::json!({
            "distance": distance,
            "time": time,
        })
    }

    fn serialize_state_info(&self, _state: &TraversalState) -> serde_json::Value {
        serde_json::json!({
            "distance_unit": self.output_distance_unit,
            "time_unit": self.output_time_unit,
        })
    }
}

fn update_state(state: &TraversalState, distance: Distance, time: Time) -> TraversalState {
    let mut updated_state = state.clone();
    updated_state[0] = state[0] + distance.into();
    updated_state[1] = state[1] + time.into();
    updated_state
}

fn get_distance_from_state(state: &TraversalState) -> Distance {
    Distance::new(state[0].0)
}

fn get_time_from_state(state: &TraversalState) -> Time {
    Time::new(state[1].0)
}

/// look up a speed from the speed table
pub fn get_speed(speed_table: &[Speed], edge_id: EdgeId) -> Result<Speed, TraversalModelError> {
    let speed: &Speed = speed_table.get(edge_id.as_usize()).ok_or(
        TraversalModelError::MissingIdInTabularCostFunction(
            format!("{}", edge_id),
            String::from("EdgeId"),
            String::from("speed table"),
        ),
    )?;
    Ok(*speed)
}

pub fn get_max_speed(speed_table: &[Speed]) -> Result<Speed, TraversalModelError> {
    let (max_speed, count) =
        speed_table
            .iter()
            .fold((Speed::ZERO, 0), |(acc_max, acc_cnt), row| {
                let next_max = if acc_max > *row { acc_max } else { *row };
                (next_max, acc_cnt + 1)
            });

    if count == 0 {
        let msg = format!("parsed {} entries for speed table", count);
        Err(TraversalModelError::BuildError(msg))
    } else if max_speed == Speed::ZERO {
        let msg = format!("max speed was zero in speed table with {} entries", count);
        Err(TraversalModelError::BuildError(msg))
    } else {
        Ok(max_speed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        property::{edge::Edge, vertex::Vertex},
        road_network::{edge_id::EdgeId, vertex_id::VertexId},
    };
    use crate::util::unit::Distance;
    use geo::coord;
    use std::path::PathBuf;

    fn mock_vertex() -> Vertex {
        Vertex {
            vertex_id: VertexId(0),
            coordinate: coord! {x: -86.67, y: 36.12},
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
        let lookup = SpeedLookupModel::new(
            &file,
            SpeedUnit::KilometersPerHour,
            None,
            Some(TimeUnit::Seconds),
        )
        .unwrap();
        let initial = lookup.initial_state();
        let v = mock_vertex();
        let e1 = mock_edge(0);
        // 100 meters @ 10kph should take 36 seconds ((0.1/10) * 3600)
        let result = lookup.traversal_cost(&v, &e1, &v, &initial).unwrap();
        let expected = 36.0;
        approx_eq(result.total_cost.into(), expected, 0.001);
        approx_eq(result.updated_state[1].into(), expected, 0.001);
    }

    #[test]
    fn test_edge_cost_lookup_with_milliseconds_time_unit() {
        let file = filepath();
        let lookup = SpeedLookupModel::new(
            &file,
            SpeedUnit::KilometersPerHour,
            None,
            Some(TimeUnit::Milliseconds),
        )
        .unwrap();
        let initial = lookup.initial_state();
        let v = mock_vertex();
        let e1 = mock_edge(0);
        // 100 meters @ 10kph should take 36,000 milliseconds ((0.1/10) * 3600000)
        let result = lookup.traversal_cost(&v, &e1, &v, &initial).unwrap();
        let expected = 36000.0;
        approx_eq(result.total_cost.into(), expected, 0.001);
        approx_eq(result.updated_state[1].into(), expected, 0.001);
    }
}
