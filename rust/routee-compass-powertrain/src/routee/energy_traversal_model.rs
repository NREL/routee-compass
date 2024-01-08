use super::energy_model_ops::get_grade;
use super::energy_model_service::EnergyModelService;
use super::vehicle::vehicle_type::{VehicleState, VehicleType};
use routee_compass_core::model::property::edge::Edge;
use routee_compass_core::model::property::vertex::Vertex;
use routee_compass_core::model::traversal::default::speed_traversal_model::get_speed;
use routee_compass_core::model::traversal::state::state_variable::StateVar;
use routee_compass_core::model::traversal::state::traversal_state::TraversalState;
use routee_compass_core::model::traversal::traversal_model::TraversalModel;
use routee_compass_core::model::traversal::traversal_model_error::TraversalModelError;
use routee_compass_core::model::unit::*;
use routee_compass_core::util::geo::haversine;
use std::sync::Arc;

pub struct EnergyTraversalModel {
    pub service: Arc<EnergyModelService>,
    pub vehicle: Arc<dyn VehicleType>,
}

impl TraversalModel for EnergyTraversalModel {
    fn initial_state(&self) -> TraversalState {
        // distance, time
        let mut initial_state = vec![StateVar(0.0), StateVar(0.0)];

        // vehicle state gets slots 2..n
        let vehicle_state = self.vehicle.initial_state();
        initial_state.extend(vehicle_state);

        initial_state
    }

    fn state_variable_names(&self) -> Vec<String> {
        let mut dims = vec![String::from("distance"), String::from("time")];
        dims.extend(self.vehicle.state_variable_names());
        dims
    }

    fn serialize_state(&self, state: &TraversalState) -> serde_json::Value {
        let distance = get_distance_from_state(state);
        let time = get_time_from_state(state);
        let vehicle_state = get_vehicle_state_from_state(state);
        let vehicle_state_summary = self.vehicle.serialize_state(vehicle_state);
        serde_json::json!({
            "distance": distance,
            "time": time,
            "vehicle": vehicle_state_summary,
        })
    }

    fn serialize_state_info(&self, state: &TraversalState) -> serde_json::Value {
        let vehicle_state = get_vehicle_state_from_state(state);
        let vehicle_state_info = self.vehicle.serialize_state_info(vehicle_state);
        serde_json::json!({
            "distance_unit": self.service.output_distance_unit,
            "time_unit": self.service.output_time_unit,
            "vehicle_info": vehicle_state_info,
        })
    }

    fn traverse_edge(
        &self,
        _src: &Vertex,
        edge: &Edge,
        _dst: &Vertex,
        state: &TraversalState,
    ) -> Result<TraversalState, TraversalModelError> {
        let distance = BASE_DISTANCE_UNIT.convert(edge.distance, self.service.output_distance_unit);
        let speed = get_speed(&self.service.speed_table, edge.edge_id)?;
        let grade = get_grade(&self.service.grade_table, edge.edge_id)?;

        let time: Time = Time::create(
            speed,
            self.service.speeds_table_speed_unit,
            distance,
            self.service.output_distance_unit,
            self.service.output_time_unit.clone(),
        )?;

        let energy_result = self.vehicle.consume_energy(
            (speed, self.service.speeds_table_speed_unit),
            (grade, self.service.grade_table_grade_unit),
            (distance, self.service.output_distance_unit),
            get_vehicle_state_from_state(state),
        )?;

        let updated_state = update_state(state, distance, time, energy_result.updated_state);
        Ok(updated_state)
    }

    fn access_edge(
        &self,
        _v1: &Vertex,
        _src: &Edge,
        _v2: &Vertex,
        _dst: &Edge,
        _v3: &Vertex,
        _state: &TraversalState,
    ) -> Result<Option<TraversalState>, TraversalModelError> {
        Ok(None)
    }

    fn estimate_traversal(
        &self,
        src: &Vertex,
        dst: &Vertex,
        state: &TraversalState,
    ) -> Result<TraversalState, TraversalModelError> {
        let distance = haversine::coord_distance(
            src.coordinate,
            dst.coordinate,
            self.service.output_distance_unit,
        )
        .map_err(TraversalModelError::NumericError)?;

        if distance == Distance::ZERO {
            return Ok(state.clone());
        }

        let time: Time = Time::create(
            self.service.max_speed,
            self.service.speeds_table_speed_unit,
            distance,
            self.service.output_distance_unit,
            self.service.output_time_unit.clone(),
        )?;

        let vehicle_state = get_vehicle_state_from_state(state);
        let best_case_result = self
            .vehicle
            .best_case_energy_state((distance, self.service.output_distance_unit), vehicle_state)?;

        let updated_state = update_state(state, distance, time, best_case_result.updated_state);
        Ok(updated_state)
    }
}

impl TryFrom<(Arc<EnergyModelService>, &serde_json::Value)> for EnergyTraversalModel {
    type Error = TraversalModelError;

    fn try_from(input: (Arc<EnergyModelService>, &serde_json::Value)) -> Result<Self, Self::Error> {
        let (service, conf) = input;

        let prediction_model_name = conf
            .get("model_name".to_string())
            .ok_or_else(|| {
                TraversalModelError::BuildError("No 'model_name' key provided in query".to_string())
            })?
            .as_str()
            .ok_or_else(|| {
                TraversalModelError::BuildError(
                    "Expected 'model_name' value to be string".to_string(),
                )
            })?
            .to_string();

        let vehicle = match service.vehicle_library.get(&prediction_model_name) {
            None => {
                let model_names: Vec<&String> = service.vehicle_library.keys().collect();
                Err(TraversalModelError::BuildError(format!(
                    "No vehicle found with model_name = '{}', try one of: {:?}",
                    prediction_model_name, model_names
                )))
            }
            Some(mr) => Ok(mr.clone()),
        }?
        .update_from_query(conf)?;

        Ok(EnergyTraversalModel { service, vehicle })
    }
}

fn update_state(
    state: &TraversalState,
    distance: Distance,
    time: Time,
    vehicle_state: VehicleState,
) -> TraversalState {
    let mut updated_state = Vec::new();

    updated_state.push(state[0] + distance.into());
    updated_state.push(state[1] + time.into());
    updated_state.extend(vehicle_state);
    updated_state
}

fn get_distance_from_state(state: &TraversalState) -> Distance {
    Distance::new(state[0].0)
}

fn get_time_from_state(state: &TraversalState) -> Time {
    Time::new(state[1].0)
}

fn get_vehicle_state_from_state(state: &TraversalState) -> &[StateVar] {
    &state[2..]
}

#[cfg(test)]
mod tests {
    use crate::routee::{
        prediction::load_prediction_model, prediction::model_type::ModelType,
        vehicle::default::ice::ICE,
    };

    use super::*;
    use geo::coord;
    use routee_compass_core::model::{
        property::{edge::Edge, vertex::Vertex},
        road_network::{edge_id::EdgeId, vertex_id::VertexId},
    };
    use std::{collections::HashMap, path::PathBuf};

    #[test]
    fn test_edge_cost_lookup_from_file() {
        let speed_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("routee")
            .join("test")
            .join("velocities.txt");
        let grade_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("routee")
            .join("test")
            .join("grades.txt");
        let model_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("routee")
            .join("test")
            .join("Toyota_Camry.bin");
        let v = Vertex {
            vertex_id: VertexId(0),
            coordinate: coord! {x: -86.67, y: 36.12},
        };
        fn mock_edge(edge_id: usize) -> Edge {
            Edge {
                edge_id: EdgeId(edge_id),
                src_vertex_id: VertexId(0),
                dst_vertex_id: VertexId(1),
                distance: Distance::new(100.0),
            }
        }
        let model_record = load_prediction_model(
            "Toyota_Camry".to_string(),
            &model_file_path,
            ModelType::Smartcore,
            SpeedUnit::MilesPerHour,
            GradeUnit::Decimal,
            EnergyRateUnit::GallonsGasolinePerMile,
            None,
            None,
            None,
        )
        .unwrap();

        let camry = ICE::new("Toyota_Camry".to_string(), model_record).unwrap();

        let mut model_library: HashMap<String, Arc<dyn VehicleType>> = HashMap::new();
        model_library.insert("Toyota_Camry".to_string(), Arc::new(camry));

        let service = EnergyModelService::new(
            &speed_file_path,
            SpeedUnit::KilometersPerHour,
            &Some(grade_file_path),
            Some(GradeUnit::Millis),
            None,
            None,
            model_library,
        )
        .unwrap();
        let arc_service = Arc::new(service);
        let conf = serde_json::json!({
            "model_name": "Toyota_Camry",
        });
        let model = EnergyTraversalModel::try_from((arc_service, &conf)).unwrap();
        let initial = model.initial_state();
        let e1 = mock_edge(0);
        // 100 meters @ 10kph should take 36 seconds ((0.1/10) * 3600)
        let result = model.traverse_edge(&v, &e1, &v, &initial).unwrap();
        println!("{:?}", result);
    }
}
