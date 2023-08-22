use uom::si;
use uom::si::f64::Velocity;

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
        units::TimeUnit,
    },
    util::fs::read_utils,
};

pub struct VelocityLookupModel {
    velocities: Vec<Velocity>,
    output_unit: TimeUnit,
}

impl VelocityLookupModel {
    pub fn from_file(
        lookup_table_filename: &String,
        output_unit: TimeUnit,
    ) -> Result<VelocityLookupModel, TraversalModelError> {
        // decodes the row into a velocity kph, and convert into internal cps
        let op = move |_idx: usize, row: String| {
            row.parse::<f64>()
                .map(|f| Velocity::new::<si::velocity::kilometer_per_hour>(f))
                .map_err(|e| {
                    let msg = format!(
                        "failure decoding velocity from lookup table: {}",
                        e.to_string()
                    );
                    std::io::Error::new(std::io::ErrorKind::InvalidData, msg)
                })
        };
        // use helper function to read the file and decode rows with the above op.
        // the resulting table has indices that are assumed EdgeIds and entries that
        // are velocities in kph.
        let velocities =
            read_utils::read_raw_file(lookup_table_filename, op, None).map_err(|e| {
                TraversalModelError::FileReadError(lookup_table_filename.clone(), e.to_string())
            })?;
        let model = VelocityLookupModel {
            velocities,
            output_unit,
        };
        Ok(model)
    }
}

impl TraversalModel for VelocityLookupModel {
    fn initial_state(&self) -> TraversalState {
        vec![StateVar(0.0)]
    }
    fn traversal_cost(
        &self,
        src: &Vertex,
        edge: &Edge,
        dst: &Vertex,
        state: &TraversalState,
    ) -> Result<TraversalResult, TraversalModelError> {
        let ff_vel = self.velocities.get(edge.edge_id.0 as usize).ok_or(
            TraversalModelError::MissingIdInTabularCostFunction(
                edge.edge_id.to_string(),
                String::from("EdgeId"),
                String::from("edge velocity lookup"),
            ),
        )?;
        let time = edge.distance.clone() / ff_vel.clone();
        let time: f64 = match self.output_unit {
            TimeUnit::Seconds => time.get::<si::time::second>().into(),
            TimeUnit::Milliseconds => time.get::<si::time::millisecond>().into(),
        };
        let mut s = state.clone();
        s[0] = s[0] + StateVar(time);
        let result = TraversalResult {
            total_cost: Cost::from(time),
            updated_state: s,
        };
        Ok(result)
    }
    fn summary(&self, state: &TraversalState) -> serde_json::Value {
        let time = state[0].0;
        let time_units = match self.output_unit {
            TimeUnit::Seconds => "seconds",
            TimeUnit::Milliseconds => "milliseconds",
        };
        serde_json::json!({
            "total_time": time,
            "units": time_units,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::cost::cost::Cost;
    use crate::model::traversal::state::state_variable::StateVar;
    use crate::model::units::{Length, Ratio, TimeUnit};
    use crate::model::{
        graph::{edge_id::EdgeId, vertex_id::VertexId},
        property::{edge::Edge, road_class::RoadClass, vertex::Vertex},
    };
    use geo::coord;
    use std::path::PathBuf;
    use uom::si;

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
            distance: Length::new::<si::length::meter>(100.0),
            grade: Ratio::new::<si::ratio::per_mille>(0.0),
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
        let output_unit = TimeUnit::Seconds;
        let lookup = VelocityLookupModel::from_file(&file, output_unit).unwrap();
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
        let output_unit = TimeUnit::Milliseconds;
        let lookup = VelocityLookupModel::from_file(&file, output_unit).unwrap();
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
