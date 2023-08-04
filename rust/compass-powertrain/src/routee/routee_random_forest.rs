use compass_core::model::traversal::function::default::velocity::edge_velocity_lookup::build_edge_velocity_lookup;
use compass_core::model::units::TimeUnit;
use compass_core::model::{
    cost::cost::Cost,
    traversal::{
        function::{cost_function_error::CostFunctionError, function::EdgeCostFunction},
        state::{search_state::StateVector, state_variable::StateVar},
        traversal_error::TraversalError,
    },
    units::Velocity,
};
use smartcore::{
    ensemble::random_forest_regressor::RandomForestRegressor, linalg::basic::matrix::DenseMatrix,
};
use uom::si;

pub fn initial_energy_state() -> StateVector {
    vec![StateVar::ZERO]
}

pub fn build_routee_random_forest(
    routee_model_path: String,
    speed_table_file: String,
) -> Result<EdgeCostFunction, CostFunctionError> {
    // load the random forest model here, similar to route prototype
    // copy that code over from prototype into this module instead of
    // referencing code directly from the compass_prototype lib

    ////////////////////////////////////////////////////////////////////////////
    // managing arguments

    // the arguments for routee powertrain include
    // grade, speed, vehicle parameters, and distance.
    // these will come from different places:

    // distance and grade come directly from
    // the Edge parameter of the EdgeCostFunction
    // speeds come from a speed lookup table
    // which must be passed as an argument to
    // the routee powertrain EdgeCostFunction builder
    // a VehicleParameters lookup table needs to be constructed
    // in the RouteE Powertrain EdgeCostFunction builder
    // an application workflow, when route powertrains are requested,
    // reify these dependencies based on this order:

    // takes speeds &EdgeCostFunction argument
    // read in routee powertrain random forest
    // user submits query including vehicle parameters, used with speeds table
    // to build an instance of routee powertrain edge lookup table
    ////////////////////////////////////////////////////////////////////////////

    // Build speed table
    let output_unit = TimeUnit::Milliseconds;
    let speed_table = build_edge_velocity_lookup(&speed_table_file, output_unit)?;
    // Load random forest binary file
    let rf_binary = std::fs::read(routee_model_path.clone())
        .map_err(|e| CostFunctionError::FileReadError(routee_model_path.clone(), e.to_string()))?;
    let rf: RandomForestRegressor<f64, f64, DenseMatrix<f64>, Vec<f64>> =
        bincode::deserialize(&rf_binary).map_err(|e| {
            CostFunctionError::FileReadError(routee_model_path.clone(), e.to_string())
        })?;

    // Build edge function
    let f: EdgeCostFunction = Box::new(move |o, e, d, s| {
        //
        // lookup routee cost, return energy cost here (instead of Cost::ZERO)
        //
        let (speed_kph, _state) = speed_table(o, e, d, s)?;
        let distance = e.distance;
        let grade = e.grade;
        let distance_mile = distance.get::<si::length::mile>();
        let grade_percent = grade.get::<si::ratio::percent>();
        let speed_mph = Velocity::new::<si::velocity::kilometer_per_hour>(speed_kph.into())
            .get::<si::velocity::mile_per_hour>();
        let x = DenseMatrix::from_2d_vec(&vec![vec![speed_mph, grade_percent]]);
        let energy_per_mile = rf.predict(&x).map_err(|e| {
            TraversalError::PredictionModel(routee_model_path.clone(), e.to_string())
        })?;
        let energy_cost = energy_per_mile[0] * distance_mile;
        let updated_state = s[0].0 + energy_cost;
        Ok((Cost::from(energy_cost), vec![StateVar(updated_state)]))
    });
    return Ok(f);
}

#[cfg(test)]
mod tests {
    use crate::routee::routee_random_forest::build_routee_random_forest;
    use compass_core::model::traversal::function::default::velocity::edge_velocity_lookup::initial_velocity_state;
    use compass_core::model::units::{Length, Ratio};
    use compass_core::model::{
        graph::{edge_id::EdgeId, vertex_id::VertexId},
        property::{edge::Edge, road_class::RoadClass, vertex::Vertex},
    };
    use geo::coord;
    use std::path::PathBuf;
    use uom::si;

    #[test]
    fn test_edge_cost_lookup_from_file() {
        let speed_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("routee")
            .join("test")
            .join("velocities.txt");
        let model_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("routee")
            .join("test")
            .join("Toyota_Camry.bin");
        let speed_file_name = speed_file_path.to_str().unwrap();
        let model_file_name = model_file_path.to_str().unwrap();
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
        let rf_predictor = build_routee_random_forest(
            String::from(model_file_name),
            String::from(speed_file_name),
        )
        .unwrap();
        let initial = initial_velocity_state();
        let e1 = mock_edge(0);
        // 100 meters @ 10kph should take 36 seconds ((0.1/10) * 3600)
        let (result_cost, result_state) = rf_predictor(&v, &e1, &v, &initial).unwrap();
    }
}
