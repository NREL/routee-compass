use compass_core::model::property::edge::Edge;
use compass_core::model::property::vertex::Vertex;
use compass_core::model::traversal::state::state_variable::StateVar;
use compass_core::model::traversal::state::traversal_state::TraversalState;
use compass_core::model::traversal::traversal_model::TraversalModel;
use compass_core::model::traversal::traversal_model_error::TraversalModelError;
use compass_core::model::traversal::traversal_result::TraversalResult;
use compass_core::util::fs::read_decoders;
use compass_core::util::geo::haversine::coord_distance_km;
use compass_core::util::unit::*;
use compass_core::{model::cost::cost::Cost, util::fs::read_utils};
use smartcore::{
    ensemble::random_forest_regressor::RandomForestRegressor, linalg::basic::matrix::DenseMatrix,
};

pub struct SmartCoreEnergyModel {
    pub speed_table: Vec<Speed>,
    pub routee_model: RandomForestRegressor<f64, f64, DenseMatrix<f64>, Vec<f64>>,
    pub energy_percent: f64,
    pub routee_model_energy_rate_unit: EnergyRateUnit,
    pub speeds_table_speed_unit: SpeedUnit,
    pub routee_model_speed_unit: SpeedUnit,
    pub minimum_energy_rate: EnergyRate,
}

impl TraversalModel for SmartCoreEnergyModel {
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
        let (energy, energy_unit) = Energy::create(
            self.minimum_energy_rate,
            self.routee_model_energy_rate_unit.clone(),
            distance,
            DistanceUnit::Kilometers,
        )?;
        Ok(Cost::from(energy))
    }
    fn traversal_cost(
        &self,
        src: &Vertex,
        edge: &Edge,
        dst: &Vertex,
        state: &TraversalState,
    ) -> Result<TraversalResult, TraversalModelError> {
        let time_unit = self.speeds_table_speed_unit.associated_time_unit();
        let time_state = unpack_time_state(state);
        let energy_state = unpack_energy_state(state);
        let speed = self.speed_table.get(edge.edge_id.as_usize()).ok_or(
            TraversalModelError::MissingIdInTabularCostFunction(
                format!("{}", edge.edge_id),
                String::from("EdgeId"),
                String::from("speed table"),
            ),
        )?;

        let time: Time = Time::create(
            *speed,
            self.speeds_table_speed_unit.clone(),
            edge.distance,
            DistanceUnit::Meters,
            time_unit.clone(),
        )?;
        let grade = edge.grade;
        let speed_routee = self
            .speeds_table_speed_unit
            .convert(*speed, self.routee_model_speed_unit.clone());
        let x = DenseMatrix::from_2d_vec(&vec![vec![speed_routee.to_f64(), grade]]);
        let y = self
            .routee_model
            .predict(&x)
            .map_err(|e| TraversalModelError::PredictionModel(e.to_string()))?;
        let energy_rate = EnergyRate::new(y[0]);
        let energy_rate_safe = if energy_rate < self.minimum_energy_rate {
            self.minimum_energy_rate
        } else {
            energy_rate
        };
        let (energy, _energy_unit) = Energy::create(
            energy_rate_safe,
            self.routee_model_energy_rate_unit.clone(),
            edge.distance,
            DistanceUnit::Meters,
        )?;

        let energy_scaled = energy * self.energy_percent;
        let energy_cost = Cost::from(energy_scaled);
        let time_scaled = time * (1.0 - self.energy_percent);
        let time_cost = Cost::from(time_scaled);
        let total_cost = energy_cost + time_cost;
        let updated_state = update_state(&state, time, energy);
        let result = TraversalResult {
            total_cost,
            updated_state,
        };
        Ok(result)
    }

    fn summary(&self, state: &TraversalState) -> serde_json::Value {
        let time = get_time_from_state(state);
        let time_unit = self.speeds_table_speed_unit.associated_time_unit();
        let energy = get_energy_from_state(state);
        let energy_unit = self.routee_model_energy_rate_unit.associated_energy_unit();
        serde_json::json!({
            "energy": energy,
            "energy_unit": energy_unit,
            "time": time,
            "time_unit": time_unit
        })
    }
}

impl SmartCoreEnergyModel {
    pub fn new(
        speed_table_path: &String,
        routee_model_path: &String,
        routee_model_energy_rate_unit: EnergyRateUnit,
        speeds_table_speed_unit: SpeedUnit,
        routee_model_speed_unit: SpeedUnit,
        energy_percent: f64,
    ) -> Result<Self, TraversalModelError> {
        // load speeds table
        let speed_table: Vec<Speed> =
            read_utils::read_raw_file(speed_table_path, read_decoders::default, None).map_err(
                |e| TraversalModelError::FileReadError(speed_table_path.clone(), e.to_string()),
            )?;

        // Load random forest binary file
        let rf_binary = std::fs::read(routee_model_path.clone()).map_err(|e| {
            TraversalModelError::FileReadError(routee_model_path.clone(), e.to_string())
        })?;
        let routee_model: RandomForestRegressor<f64, f64, DenseMatrix<f64>, Vec<f64>> =
            bincode::deserialize(&rf_binary).map_err(|e| {
                TraversalModelError::FileReadError(routee_model_path.clone(), e.to_string())
            })?;

        // sweep a fixed set of speed and grade values to find the minimum energy per mile rate from the incoming rf model
        let mut minimum_energy_rate = std::f64::MAX;
        let start_time = std::time::Instant::now();

        for speed_mph in 1..100 {
            for grade_percent in -20..20 {
                let x =
                    DenseMatrix::from_2d_vec(&vec![vec![speed_mph as f64, grade_percent as f64]]);
                let energy_per_mile = routee_model
                    .predict(&x)
                    .map_err(|e| TraversalModelError::PredictionModel(e.to_string()))?;
                if energy_per_mile[0] < minimum_energy_rate {
                    minimum_energy_rate = energy_per_mile[0];
                }
            }
        }

        let end_time = std::time::Instant::now();
        let search_time = end_time - start_time;

        log::debug!(
            "found minimum_energy_per_mile: {} for {} in {} milliseconds",
            minimum_energy_rate,
            routee_model_path,
            search_time.as_millis()
        );

        Ok(SmartCoreEnergyModel {
            speed_table,
            routee_model,
            energy_percent,
            routee_model_energy_rate_unit,
            speeds_table_speed_unit,
            routee_model_speed_unit,
            minimum_energy_rate: EnergyRate::new(minimum_energy_rate),
        })
    }
}

fn unpack_time_state(state: &TraversalState) -> TraversalState {
    return vec![state[0]];
}

fn unpack_energy_state(state: &TraversalState) -> TraversalState {
    return vec![state[1]];
}

fn update_state(state: &TraversalState, time: Time, energy: Energy) -> TraversalState {
    let mut updated_state = state.clone();
    updated_state[0] = state[0] + time.into();
    updated_state[1] = state[1] + energy.into();
    return updated_state;
}

fn get_time_from_state(state: &TraversalState) -> Time {
    return Time::new(state[0].0);
}

fn get_energy_from_state(state: &TraversalState) -> Energy {
    return Energy::new(state[1].0);
}

fn pack_state(time_state: &TraversalState, energy_state: &TraversalState) -> TraversalState {
    return vec![time_state[0], energy_state[0]];
}

#[cfg(test)]
mod tests {
    use super::*;
    use compass_core::model::{
        graph::{edge_id::EdgeId, vertex_id::VertexId},
        property::{edge::Edge, road_class::RoadClass, vertex::Vertex},
    };
    use geo::coord;
    use std::path::PathBuf;

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
                distance: Distance::new(100.0),
                grade: 0.0,
            };
        }
        let speed_file = String::from(speed_file_name);
        let routee_model_path = String::from(model_file_name);
        let rf_predictor = SmartCoreEnergyModel::new(
            &speed_file,
            &routee_model_path,
            EnergyRateUnit::GallonsGasolinePerMile,
            SpeedUnit::KilometersPerHour,
            SpeedUnit::MilesPerHour,
            1.0,
        )
        .unwrap();
        let initial = rf_predictor.initial_state();
        let e1 = mock_edge(0);
        // 100 meters @ 10kph should take 36 seconds ((0.1/10) * 3600)
        let result = rf_predictor.traversal_cost(&v, &e1, &v, &initial).unwrap();
    }
}
