use std::sync::Arc;

use compass_core::util::fs::read_decoders;
use compass_core::{model::cost::cost::Cost, util::fs::read_utils};
use compass_core::model::property::edge::Edge;
use compass_core::model::property::vertex::Vertex;
use compass_core::model::traversal::default::velocity_lookup::VelocityLookupModel;
use compass_core::model::traversal::state::state_variable::StateVar;
use compass_core::model::traversal::state::traversal_state::TraversalState;
use compass_core::model::traversal::traversal_model::TraversalModel;
use compass_core::model::traversal::traversal_model_error::TraversalModelError;
use compass_core::model::traversal::traversal_result::TraversalResult;
use compass_core::util::geo::haversine::coord_distance_km;
use compass_core::util::unit::*;
use smartcore::{
    ensemble::random_forest_regressor::RandomForestRegressor, linalg::basic::matrix::DenseMatrix,
};
use uom::si;

pub struct RouteERandomForestModel {
    pub speed_table: Vec<Speed>,
    pub routee_model: RandomForestRegressor<f64, f64, DenseMatrix<f64>, Vec<f64>>,
    pub energy_percent: f64,
    pub energy_unit: EnergyUnit,
    pub speeds_table_speed_unit: SpeedUnit,
    pub routee_model_speed_unit: SpeedUnit,
    pub minimum_energy_per_mile: f64,
}

impl TraversalModel for RouteERandomForestModel {
    fn initial_state(&self) -> TraversalState {
        // time, energy
        vec![StateVar(0.0), StateVar(0.0)]
    }
    fn cost_estimate(
        &self,
        src: &Vertex,
        dst: &Vertex,
        _state: &TraversalState,
    ) -> Result<Cost, TraversalModelError> {
        let distance = coord_distance_km(src.coordinate, dst.coordinate)
            .map_err(TraversalModelError::NumericError)?;
        let distance_miles = DistanceUnit::Kilometers.convert(distance, min_energy_unit);
        let minimum_energy = self.minimum_energy_per_mile * distance_miles;
        Ok(Cost::from(minimum_energy))
    }
    fn traversal_cost(
        &self,
        src: &Vertex,
        edge: &Edge,
        dst: &Vertex,
        state: &TraversalState,
    ) -> Result<TraversalResult, TraversalModelError> {
        let time_unit = TimeUnit::Seconds;
        let time_state = unpack_time_state(state);
        let energy_state = unpack_energy_state(state);
        let speed = self.speed_table
            .get(edge.edge_id)
            .ok_or(TraversalModelError::MissingIdInTabularCostFunction(
                String::from(edge.edge_id), 
                String::from("EdgeId"), 
                String::from("speed table")))?;
        let travel_time_state = get_time_value_from_time_state(&time_result.updated_state);
        let time: Time = time_unit.calculate_time(speed, self.speeds_table_speed_unit, edge.distance, DistanceUnit::Meters);
        // let speed_result = self.velocity_model.traversal_cost(src, edge, dst, state)?;
        // let speed_kph: f64 = speed_result.total_cost.into();
        // let distance = edge.distance;
        let grade = edge.grade;
        // let distance_mile = distance.get::<si::length::mile>();
        // let grade_percent = grade.get::<si::ratio::percent>();
        // let speed_mph = speed_kph.get::<si::velocity::mile_per_hour>();
        let speed_routee = self.speeds_table_speed_unit.convert(speed, self.routee_model_speed_unit);
        let x = DenseMatrix::from_2d_vec(&vec![vec![speed_routee, grade]]);
        let y = self
            .routee_model
            .predict(&x)
            .map_err(|e| TraversalModelError::PredictionModel(e.to_string()))?;
        let energy = Energy(y[0]);
        
        // todo:
        // - currently stepping through and replacing code using uom with our homemade units lib
        // - there's a few files with compile errors, but maybe get to the end of this file first
        // - we need Energy and EnergyRate units, with a EnergyRate.calculate_energy() method taking energy_rate and distance

        let mut energy_cost = y[0] * distance_mile;
        // set cost to zero if it's negative since we can't currently handle negative costs
        energy_cost = if energy_cost < 0.000001 { 0.000001 } else { energy_cost };

        let mut updated_state = state.clone();
        updated_state[0] = state[0] + StateVar(energy_cost);
        let result = TraversalResult {
            total_cost: Cost::from(energy_cost),
            updated_state,
        };
        Ok(result)
    }
    fn summary(&self, state: &TraversalState) -> serde_json::Value {
        let total_time = state[0]
        let total_energy = state[1].0;
        serde_json::json!({
            "total_energy": total_energy,
            "energy_units": self.energy_unit.to_string()
        })
    }
}

impl RouteERandomForestModel {
    pub fn new(
        speed_table_path: &String,
        routee_model_path: &String,
        speeds_table_speed_unit: SpeedUnit,
        routee_model_speed_unit: SpeedUnit,
        model_energy_unit: EnergyUnit,
        energy_percent: f64,
    ) -> Result<Self, TraversalModelError> {
        // load speeds table
        let speed_table: Vec<Speed> =
        read_utils::read_raw_file(speed_table_path, read_decoders::default, None).map_err(|e| {
            TraversalModelError::FileReadError(lookup_table_filename.clone(), e.to_string())
        })?;
        
        // Load random forest binary file
        let rf_binary = std::fs::read(routee_model_path.clone()).map_err(|e| {
            TraversalModelError::FileReadError(routee_model_path.clone(), e.to_string())
        })?;
        let routee_model: RandomForestRegressor<f64, f64, DenseMatrix<f64>, Vec<f64>> =
            bincode::deserialize(&rf_binary).map_err(|e| {
                TraversalModelError::FileReadError(routee_model_path.clone(), e.to_string())
            })?;

        // sweep a fixed set of speed and grade values to find the minimum energy per mile rate from the incoming rf model
        let mut minimum_energy_per_mile = std::f64::MAX;

        let start_time = std::time::Instant::now();

        for speed_mph in 1..100 {
            for grade_percent in -20..20 {
                let x =
                    DenseMatrix::from_2d_vec(&vec![vec![speed_mph as f64, grade_percent as f64]]);
                let energy_per_mile = rf
                    .predict(&x)
                    .map_err(|e| TraversalModelError::PredictionModel(e.to_string()))?;
                if energy_per_mile[0] < minimum_energy_per_mile {
                    minimum_energy_per_mile = energy_per_mile[0];
                }
            }
        }

        let end_time = std::time::Instant::now();
        let search_time = end_time - start_time;

        log::debug!(
            "found minimum_energy_per_mile: {} for {} in {} milliseconds",
            minimum_energy_per_mile,
            routee_model_path,
            search_time.as_millis()
        );

        Ok(RouteERandomForestModel {
            speed_table,
            routee_model,
            energy_percent,
            energy_unit,
            speeds_table_speed_unit,
            routee_model_speed_unit,
            minimum_energy_per_mile,
        })
    }

    // pub fn new_w_speed_file(
    //     speed_file: &String,
    //     routee_model_path: &String,
    //     energy_percent: f64,
    //     time_unit: TimeUnit,
    //     energy_rate_unit: EnergyUnit,
    // ) -> Result<Self, TraversalModelError> {
    //     let velocity_model = VelocityLookupModel::from_file(&speed_file, time_unit.clone())?;
    //     Self::new(
    //         Arc::new(velocity_model),
    //         routee_model_path,
    //         energy_rate_unit,
    //         time_unit,
    //         energy_percent,
    //     )
    // }
}

fn unpack_time_state(state: &TraversalState) -> TraversalState {
    return vec![state[0]];
}

fn unpack_energy_state(state: &TraversalState) -> TraversalState {
    return vec![state[1]];
}

fn get_time_value_from_time_state(state: &TraversalState) -> Time {
    Time::new::<si::time::second>(state[0].0)
}

fn pack_state(time_state: &TraversalState, energy_state: &TraversalState) -> TraversalState {
    return vec![time_state[0], energy_state[0]];
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let speed_file = String::from(speed_file_name);
        let routee_model_path = String::from(model_file_name);
        let rf_predictor = RouteERandomForestModel::new_w_speed_file(
            &speed_file,
            &routee_model_path,
            1.0,
            TimeUnit::Seconds,
            EnergyUnit::GallonsGasoline,
        )
        .unwrap();
        let initial = rf_predictor.initial_state();
        let e1 = mock_edge(0);
        // 100 meters @ 10kph should take 36 seconds ((0.1/10) * 3600)
        let result = rf_predictor.traversal_cost(&v, &e1, &v, &initial).unwrap();
    }
}
