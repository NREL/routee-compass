use crate::model::graph::edge_id::EdgeId;
use crate::util::fs::read_decoders;
use crate::util::geo::haversine;
use crate::util::unit::{DistanceUnit, BASE_SPEED_UNIT};
use crate::util::unit::{SpeedUnit, Time, TimeUnit, BASE_DISTANCE_UNIT, BASE_TIME_UNIT};
use crate::{
    model::{
        cost::cost::Cost,
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

pub struct SpeedLookupModel {
    speed_table: Vec<Speed>,
    speed_unit: SpeedUnit,
    output_time_unit: TimeUnit,
    output_distance_unit: DistanceUnit,
    max_speed: Speed,
}

impl SpeedLookupModel {
    pub fn new(
        speed_table_path: &String,
        speed_unit_opt: Option<SpeedUnit>,
        output_distance_unit_opt: Option<DistanceUnit>,
        output_time_unit_opt: Option<TimeUnit>,
    ) -> Result<SpeedLookupModel, TraversalModelError> {
        let speed_table: Vec<Speed> =
            read_utils::read_raw_file(speed_table_path, read_decoders::default, None).map_err(
                |e| TraversalModelError::FileReadError(speed_table_path.clone(), e.to_string()),
            )?;

        let max_speed = get_max_speed(&speed_table)?;
        let speed_unit = match speed_unit_opt {
            Some(su) => {
                log::info!("speed table configured with speeds in {}", su.clone());
                su.clone()
            }
            None => {
                log::info!(
                    "no speed unit provided for speed table, using default of {}",
                    BASE_SPEED_UNIT
                );
                BASE_SPEED_UNIT
            }
        };
        let output_distance_unit = match output_distance_unit_opt {
            Some(du) => {
                log::info!("speed model configured with output units in {}", du.clone());
                du.clone()
            }
            None => {
                log::info!(
                    "no distance unit provided for speed model, using default of {}",
                    BASE_DISTANCE_UNIT
                );
                BASE_DISTANCE_UNIT
            }
        };
        let output_time_unit = match output_time_unit_opt {
            Some(tu) => {
                log::info!("speed model configured with output units in {}", tu.clone());
                tu.clone()
            }
            None => {
                log::info!(
                    "no time unit provided for speed model, using default of {}",
                    BASE_TIME_UNIT
                );
                BASE_TIME_UNIT
            }
        };
        let model = SpeedLookupModel {
            speed_table,
            output_distance_unit,
            output_time_unit,
            speed_unit,
            max_speed: max_speed.clone(),
        };
        return Ok(model);
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
        let speed = get_speed(&self.speed_table, edge.edge_id)?;
        let time = Time::create(
            speed.clone(),
            self.speed_unit.clone(),
            edge.distance,
            BASE_DISTANCE_UNIT,
            self.output_time_unit.clone(),
        )?;

        let mut s = state.clone();
        s[0] = s[0] + StateVar::from(edge.distance);
        s[1] = s[1] + StateVar::from(time);
        let result = TraversalResult {
            total_cost: Cost::from(time),
            updated_state: s,
        };
        Ok(result)
    }

    fn cost_estimate(
        &self,
        src: &Vertex,
        dst: &Vertex,
        _state: &TraversalState,
    ) -> Result<Cost, TraversalModelError> {
        let distance = haversine::coord_distance_meters(src.coordinate, dst.coordinate)
            .map_err(TraversalModelError::NumericError)?;
        let time = Time::create(
            self.max_speed,
            self.speed_unit.clone(),
            distance,
            DistanceUnit::Meters,
            self.output_time_unit.clone(),
        )?;
        let time_output: Time = BASE_TIME_UNIT.convert(time, self.output_time_unit.clone());
        Ok(Cost::from(time_output))
    }

    fn summary(&self, state: &TraversalState) -> serde_json::Value {
        let time = state[1].0;
        let distance = state[1].0;
        serde_json::json!({
            "distance": distance,
            "distance_unit": self.output_distance_unit,
            "time": time,
            "time_unit": self.output_time_unit,
        })
    }
}

/// look up a speed from the speed table
pub fn get_speed(speed_table: &Vec<Speed>, edge_id: EdgeId) -> Result<Speed, TraversalModelError> {
    let speed: &Speed = speed_table.get(edge_id.as_usize()).ok_or(
        TraversalModelError::MissingIdInTabularCostFunction(
            format!("{}", edge_id),
            String::from("EdgeId"),
            String::from("speed table"),
        ),
    )?;
    Ok(*speed)
}

pub fn get_max_speed(speed_table: &Vec<Speed>) -> Result<Speed, TraversalModelError> {
    let (max_speed, count) =
        speed_table
            .iter()
            .fold((Speed::ZERO, 0), |(acc_max, acc_cnt), row| {
                let next_max = if acc_max > *row { acc_max } else { row.clone() };
                (next_max, acc_cnt + 1)
            });

    if count == 0 {
        let msg = format!("parsed {} entries for speed table", count);
        return Err(TraversalModelError::BuildError(msg));
    } else if max_speed == Speed::ZERO {
        let msg = format!("max speed was zero in speed table with {} entries", count);
        return Err(TraversalModelError::BuildError(msg));
    } else {
        return Ok(max_speed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{model::{
        graph::{edge_id::EdgeId, vertex_id::VertexId},
        property::{edge::Edge, road_class::RoadClass, vertex::Vertex},
    }, util::unit::Grade};
    use crate::util::unit::Distance;
    use geo::coord;
    use std::path::PathBuf;

    fn mock_vertex() -> Vertex {
        return Vertex {
            vertex_id: VertexId(0),
            coordinate: coord! {x: -86.67, y: 36.12},
        };
    }
    fn mock_edge(edge_id: usize) -> Edge {
        return Edge {
            edge_id: EdgeId(edge_id as u64),
            src_vertex_id: VertexId(0),
            dst_vertex_id: VertexId(1),
            road_class: RoadClass(2),
            distance: Distance::new(100.0),
            grade: Grade::ZERO,
        };
    }
    fn filepath() -> String {
        let filepath = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("model")
            .join("traversal")
            .join("default")
            .join("test")
            .join("velocities.txt");
        let filename = filepath.to_str().unwrap().to_owned();
        return filename;
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
            Some(SpeedUnit::KilometersPerHour),
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
        approx_eq(result.updated_state[0].into(), expected, 0.001);
    }

    #[test]
    fn test_edge_cost_lookup_with_milliseconds_time_unit() {
        let file = filepath();
        let lookup = SpeedLookupModel::new(
            &file,
            Some(SpeedUnit::KilometersPerHour),
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
        approx_eq(result.updated_state[0].into(), expected, 0.001);
    }
}
