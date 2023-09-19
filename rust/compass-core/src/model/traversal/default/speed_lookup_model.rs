use crate::util::fs::read_decoders;
use crate::util::geo::haversine::coord_distance_km;
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
    max_speed: Speed,
}

impl SpeedLookupModel {
    pub fn new(
        speed_table_path: &String,
        speed_unit: SpeedUnit,
        output_time_unit_opt: Option<TimeUnit>,
    ) -> Result<SpeedLookupModel, TraversalModelError> {
        let speed_table: Vec<Speed> =
            read_utils::read_raw_file(speed_table_path, read_decoders::default, None).map_err(
                |e| TraversalModelError::FileReadError(speed_table_path.clone(), e.to_string()),
            )?;

        let (max_speed, count) =
            speed_table
                .iter()
                .fold((Speed::ZERO, 0), |(acc_max, acc_cnt), row| {
                    let next_max = if acc_max > *row { acc_max } else { row.clone() };
                    (next_max, acc_cnt + 1)
                });

        if count == 0 {
            let msg = format!(
                "parsed {} entries for speed table {}",
                count, speed_table_path
            );
            return Err(TraversalModelError::BuildError(msg));
        } else if max_speed == Speed::ZERO {
            let msg = format!(
                "max speed was zero in speed table {} with {} entries",
                speed_table_path, count
            );
            return Err(TraversalModelError::BuildError(msg));
        } else {
            let output_time_unit =
                output_time_unit_opt.unwrap_or(speed_unit.associated_time_unit());
            let model = SpeedLookupModel {
                speed_table,
                output_time_unit,
                speed_unit,
                max_speed: max_speed.clone(),
            };
            return Ok(model);
        }
    }
}

impl TraversalModel for SpeedLookupModel {
    fn initial_state(&self) -> TraversalState {
        vec![StateVar(0.0)]
    }
    fn traversal_cost(
        &self,
        _src: &Vertex,
        edge: &Edge,
        _dst: &Vertex,
        state: &TraversalState,
    ) -> Result<TraversalResult, TraversalModelError> {
        let speed = self
            .speed_table
            .get(edge.edge_id.0 as usize)
            .ok_or_else(|| {
                TraversalModelError::MissingIdInTabularCostFunction(
                    edge.edge_id.to_string(),
                    String::from("EdgeId"),
                    String::from("edge velocity lookup"),
                )
            })?;
        let time = Time::create(
            speed.clone(),
            self.speed_unit.clone(),
            edge.distance,
            BASE_DISTANCE_UNIT,
            self.output_time_unit.clone(),
        )?;

        let time_output: Time = BASE_TIME_UNIT.convert(time, self.output_time_unit.clone());
        let mut s = state.clone();
        s[0] = s[0] + StateVar::from(time_output);
        let result = TraversalResult {
            total_cost: Cost::from(time_output),
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
        let distance = coord_distance_km(src.coordinate, dst.coordinate)
            .map_err(TraversalModelError::NumericError)?;
        let time = Time::create(
            self.max_speed,
            self.speed_unit.clone(),
            distance,
            BASE_DISTANCE_UNIT,
            self.output_time_unit.clone(),
        )?;
        let time_output: Time = BASE_TIME_UNIT.convert(time, self.output_time_unit.clone());
        Ok(Cost::from(time_output))
    }
    fn summary(&self, state: &TraversalState) -> serde_json::Value {
        let time = state[0].0;
        serde_json::json!({
            "time": time,
            "time_unit": self.output_time_unit,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::cost::cost::Cost;
    use crate::model::traversal::state::state_variable::StateVar;
    use crate::model::{
        graph::{edge_id::EdgeId, vertex_id::VertexId},
        property::{edge::Edge, road_class::RoadClass, vertex::Vertex},
    };
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
            grade: 0.0,
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

    #[test]
    fn test_edge_cost_lookup_with_seconds_time_unit() {
        let file = filepath();
        let lookup = SpeedLookupModel::new(&file, SpeedUnit::MetersPerSecond, None).unwrap();
        let initial = lookup.initial_state();
        let v = mock_vertex();
        let e1 = mock_edge(0);
        // 100 meters @ 10kph should take 36 seconds ((0.1/10) * 3600)
        let result = lookup.traversal_cost(&v, &e1, &v, &initial).unwrap();
        let expected = 36.0;
        assert_eq!(result.total_cost, Cost::from(expected));
        assert_eq!(result.updated_state, vec![StateVar(expected)]);
    }

    #[test]
    fn test_edge_cost_lookup_with_milliseconds_time_unit() {
        let file = filepath();
        let lookup = SpeedLookupModel::new(&file, SpeedUnit::MetersPerSecond, None).unwrap();
        let initial = lookup.initial_state();
        let v = mock_vertex();
        let e1 = mock_edge(0);
        // 100 meters @ 10kph should take 36,000 milliseconds ((0.1/10) * 3600000)
        let result = lookup.traversal_cost(&v, &e1, &v, &initial).unwrap();
        let expected = 36000.0;
        assert_eq!(result.total_cost, Cost::from(expected));
        assert_eq!(result.updated_state, vec![StateVar(expected)]);
    }
}
