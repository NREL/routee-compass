use std::collections::HashSet;

use crate::model::property::edge::Edge;
use crate::model::property::vertex::Vertex;
use crate::model::traversal::function::cost_function_error::CostFunctionError;
use crate::model::traversal::function::function::EdgeCostFunction;
use crate::model::traversal::state::search_state::StateVector;
use crate::model::traversal::traversal_error::TraversalError;
use crate::model::units::Velocity;
use crate::model::{cost::cost::Cost, traversal::state::state_variable::StateVar};
use crate::util::fs::read_utils;
use uom::si;
use uom::si::velocity::kilometer_per_hour;

/// implements a lookup traversal cost function where a table has one velocity
/// value per edge.
/// given some input file with |E| rows, each row has a velocity in KPH. this
/// velocity may be free flow velocity, average velocity, or other.
///
/// time unit is used to set the output unit for the cost function, which should
/// be one of {ms, sec}
pub fn build_edge_velocity_lookup(
    lookup_table_filename: &String,
    output_unit: &String,
) -> Result<EdgeCostFunction, CostFunctionError> {
    let output_units = HashSet::from([String::from("ms"), String::from("sec")]);
    if !output_units.contains(output_unit) {
        return Err(CostFunctionError::ConfigurationError(format!(
            "unknown time unit {} must be one of {:?}",
            output_unit, output_units
        )));
    }

    // decodes the row into a velocity kph, and convert into internal cps
    let op = move |_idx: usize, row: String| {
        row.parse::<f64>()
            .map(|f| Velocity::new::<kilometer_per_hour>(f))
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
    let table = read_utils::read_raw_file(lookup_table_filename, op).map_err(|e| {
        CostFunctionError::FileReadError(
            format!(
                "failure reading table from file {}",
                lookup_table_filename.clone()
            ),
            e.to_string(),
        )
    })?;

    let output_unit_cloned = output_unit.clone();
    let ffcf: EdgeCostFunction = Box::new(
        move |_src: &Vertex, edge: &Edge, _dst: &Vertex, state: &StateVector| {
            let ff_vel = table.get(edge.edge_id.0 as usize).ok_or(
                TraversalError::MissingIdInTabularCostFunction(
                    edge.edge_id.to_string(),
                    String::from("EdgeId"),
                    String::from("edge velocity lookup"),
                ),
            )?;
            let time = edge.distance.clone() / ff_vel.clone();
            let time: f64 = if output_unit_cloned == "sec" {
                time.get::<si::time::second>().into()
            } else {
                time.get::<si::time::millisecond>().into()
            };
            let mut s = state.to_vec();
            s[0] = s[0] + StateVar(time);
            Ok((Cost::from(time), s))
        },
    );
    return Ok(ffcf);
}

/// starting state for a free flow search
pub fn initial_velocity_state() -> StateVector {
    vec![StateVar::ZERO]
}

#[cfg(test)]
mod tests {
    use super::{build_edge_velocity_lookup, initial_velocity_state};
    use crate::model::cost::cost::Cost;
    use crate::model::traversal::state::state_variable::StateVar;
    use crate::model::units::{Length, Ratio};
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
            .join("function")
            .join("default")
            .join("velocity")
            .join("test")
            .join("velocities.txt");
        let filename = filepath.to_str().unwrap().to_owned();
        return filename;
    }

    #[test]
    fn test_edge_cost_lookup_with_seconds_time_unit() {
        let file = filepath();
        let output_unit = String::from("sec");
        let lookup = build_edge_velocity_lookup(&file, &output_unit).unwrap();
        let initial = initial_velocity_state();
        let v = mock_vertex();
        let e1 = mock_edge(0);
        // 100 meters @ 10kph should take 36 seconds ((0.1/10) * 3600)
        let (result_cost, result_state) = lookup(&v, &e1, &v, &initial).unwrap();
        let expected = 36.0;
        assert_eq!(result_cost, Cost::from(expected));
        assert_eq!(result_state, vec![StateVar(expected)]);
    }

    #[test]
    fn test_edge_cost_lookup_with_milliseconds_time_unit() {
        let file = filepath();
        let output_unit = String::from("ms");
        let lookup = build_edge_velocity_lookup(&file, &output_unit).unwrap();
        let initial = initial_velocity_state();
        let v = mock_vertex();
        let e1 = mock_edge(0);
        // 100 meters @ 10kph should take 36,000 milliseconds ((0.1/10) * 3600000)
        let (result_cost, result_state) = lookup(&v, &e1, &v, &initial).unwrap();
        let expected = 36000.0;
        assert_eq!(result_cost, Cost::from(expected));
        assert_eq!(result_state, vec![StateVar(expected)]);
    }
}
