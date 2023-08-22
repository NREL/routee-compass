use std::sync::Arc;

use compass_core::model::property::edge::Edge;
use compass_core::model::property::vertex::Vertex;
use compass_core::model::traversal::default::velocity_lookup::VelocityLookupModel;
use compass_core::model::traversal::state::state_variable::StateVar;
use compass_core::model::traversal::state::traversal_state::TraversalState;
use compass_core::model::traversal::traversal_model::TraversalModel;
use compass_core::model::traversal::traversal_model_error::TraversalModelError;
use compass_core::model::traversal::traversal_result::TraversalResult;
use compass_core::model::units::TimeUnit;
use compass_core::model::{cost::cost::Cost, units::Velocity};
use smartcore::{
    ensemble::random_forest_regressor::RandomForestRegressor, linalg::basic::matrix::DenseMatrix,
};
use uom::si;

pub struct RouteERandomForestModel {
    pub velocity_model: Arc<VelocityLookupModel>,
    pub routee_model: RandomForestRegressor<f64, f64, DenseMatrix<f64>, Vec<f64>>,
}

impl TraversalModel for RouteERandomForestModel {
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
        let speed_result = self.velocity_model.traversal_cost(src, edge, dst, state)?;
        let speed_kph: f64 = speed_result.total_cost.into();
        let distance = edge.distance;
        let grade = edge.grade;
        let distance_mile = distance.get::<si::length::mile>();
        let grade_percent = grade.get::<si::ratio::percent>();
        let speed_mph = Velocity::new::<si::velocity::kilometer_per_hour>(speed_kph.into())
            .get::<si::velocity::mile_per_hour>();
        let x = DenseMatrix::from_2d_vec(&vec![vec![speed_mph, grade_percent]]);
        let energy_per_mile = self
            .routee_model
            .predict(&x)
            .map_err(|e| TraversalModelError::PredictionModel(e.to_string()))?;
        let energy_cost = energy_per_mile[0] * distance_mile;
        let mut updated_state = state.clone();
        updated_state[0] = state[0] + StateVar(energy_cost);
        let result = TraversalResult {
            total_cost: Cost::from(energy_cost),
            updated_state,
        };
        Ok(result)
    }
    fn summary(&self, state: &TraversalState) -> serde_json::Value {
        let total_energy = state[0].0;
        serde_json::json!({
            "total_energy": total_energy,
            "energy_units": "gallons_gasoline"
        })
    }
}

impl RouteERandomForestModel {
    pub fn new(
        velocity_model: Arc<VelocityLookupModel>,
        routee_model_path: String,
    ) -> Result<Self, TraversalModelError> {
        // Load random forest binary file
        let rf_binary = std::fs::read(routee_model_path.clone()).map_err(|e| {
            TraversalModelError::FileReadError(routee_model_path.clone(), e.to_string())
        })?;
        let rf: RandomForestRegressor<f64, f64, DenseMatrix<f64>, Vec<f64>> =
            bincode::deserialize(&rf_binary).map_err(|e| {
                TraversalModelError::FileReadError(routee_model_path.clone(), e.to_string())
            })?;

        Ok(RouteERandomForestModel {
            velocity_model,
            routee_model: rf,
        })
    }

    pub fn new_w_speed_file(
        speed_file: String,
        routee_model_path: String,
        time_unit: TimeUnit,
    ) -> Result<Self, TraversalModelError> {
        let velocity_model = VelocityLookupModel::from_file(&speed_file, time_unit)?;
        Self::new(Arc::new(velocity_model), routee_model_path)
    }
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
        let rf_predictor = RouteERandomForestModel::new_w_speed_file(
            String::from(speed_file_name),
            String::from(model_file_name),
            TimeUnit::Seconds,
        )
        .unwrap();
        let initial = rf_predictor.initial_state();
        let e1 = mock_edge(0);
        // 100 meters @ 10kph should take 36 seconds ((0.1/10) * 3600)
        let result = rf_predictor.traversal_cost(&v, &e1, &v, &initial).unwrap();
    }
}
