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
pub fn build_edge_velocity_lookup(
    lookup_table_filename: String,
) -> Result<EdgeCostFunction, CostFunctionError> {
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
    let table = read_utils::read_raw_file(&lookup_table_filename, op).map_err(|e| {
        CostFunctionError::FileReadError(
            format!(
                "failure reading table from file {}",
                lookup_table_filename.clone()
            ),
            e.to_string(),
        )
    })?;

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
            let seconds: f64 = time.get::<si::time::second>().into();
            let mut s = state.to_vec();
            s[0] = s[0] + StateVar(seconds);
            Ok((Cost::from_f64(seconds), s))
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

    #[test]
    fn test_edge_cost_lookup_from_file() {
        let filepath = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("model")
            .join("traversal")
            .join("function")
            .join("default")
            .join("velocity")
            .join("test")
            .join("velocities.txt");
        let filename = filepath.to_str().unwrap();
        let v = Vertex {
            vertex_id: VertexId(0),
            coordinate: coord! {x: -86.67, y: 36.12},
        };
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
        let lookup = build_edge_velocity_lookup(String::from(filename)).unwrap();
        let initial = initial_velocity_state();
        let e1 = mock_edge(0);
        // 100 meters @ 10kph should take 36 seconds ((0.1/10) * 3600)
        let (result_cost, result_state) = lookup(&v, &e1, &v, &initial).unwrap();
        let expected = 36.0;
        assert_eq!(result_cost, Cost::from_f64(expected));
        assert_eq!(result_state, vec![StateVar(expected)]);
    }
}
