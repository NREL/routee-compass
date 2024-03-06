use super::energy_model_ops::get_grade;
use super::energy_model_service::EnergyModelService;
use super::vehicle::vehicle_type::VehicleType;
use routee_compass_core::model::property::edge::Edge;
use routee_compass_core::model::property::vertex::Vertex;
use routee_compass_core::model::state::state_model::StateModel;
use routee_compass_core::model::traversal::state::state_variable::StateVar;
use routee_compass_core::model::traversal::state::traversal_state::TraversalState;
use routee_compass_core::model::traversal::traversal_model::TraversalModel;
use routee_compass_core::model::traversal::traversal_model_error::TraversalModelError;
use routee_compass_core::model::unit::*;
use routee_compass_core::util::geo::haversine;
use routee_compass_core::util::serde::serde_json_extension::SerdeJsonExtension;
use std::collections::HashMap;
use std::sync::Arc;

pub struct EnergyTraversalModel {
    pub energy_model_service: Arc<EnergyModelService>,
    pub time_model: Arc<dyn TraversalModel>,
    pub vehicle: Arc<dyn VehicleType>,
    pub state_model: Arc<StateModel>,
}

impl TraversalModel for EnergyTraversalModel {
    fn traverse_edge(
        &self,
        src: &Vertex,
        edge: &Edge,
        dst: &Vertex,
        state: &[StateVar],
    ) -> Result<TraversalState, TraversalModelError> {
        let distance =
            BASE_DISTANCE_UNIT.convert(edge.distance, self.energy_model_service.distance_unit);

        // perform time traversal
        // let time_state = &state[0..self.vehicle_state_index];
        // let vehicle_state = &state[self.vehicle_state_index..];
        let mut updated_state = self.time_model.traverse_edge(src, edge, dst, state)?;
        let time_delta_var = self.state_model.get_delta(state, &updated_state, "time")?;
        let time_delta = Time::new(time_delta_var.0);
        // let time_next = self.time_model.get_state_variable("time", &updated_state)?;
        // let time_prev = self.get_state_variable("time", state)?;
        // let time_delta: Time = Time::new(time_next.0 - time_prev.0);

        // perform vehicle energy traversal
        let grade = get_grade(&self.energy_model_service.grade_table, edge.edge_id)?;
        let speed = Speed::from((distance, time_delta));
        let energy_result = self.vehicle.consume_energy(
            (speed, self.energy_model_service.time_model_speed_unit),
            (grade, self.energy_model_service.grade_table_grade_unit),
            (distance, self.energy_model_service.distance_unit),
            &updated_state,
        )?;

        updated_state.extend(energy_result.updated_state);
        Ok(updated_state)
    }

    fn access_edge(
        &self,
        v1: &Vertex,
        src: &Edge,
        v2: &Vertex,
        dst: &Edge,
        v3: &Vertex,
        state: &[StateVar],
    ) -> Result<Option<TraversalState>, TraversalModelError> {
        // defer access updates to time model
        self.time_model.access_edge(v1, src, v2, dst, v3, state)
        // match self.energy_model_service.headings_table.as_deref() {
        //     None => Ok(None),
        //     Some(headings_table) => {
        // let src_heading = get_headings(headings_table, src.edge_id)?;
        // let dst_heading = get_headings(headings_table, dst.edge_id)?;
        // let angle = src_heading.next_edge_angle(&dst_heading);
        // let turn = Turn::from_angle(angle)?;
        // let time_cost = match turn {
        //     Turn::NoTurn => {
        //         // no penalty for straight
        //         Time::new(0.0)
        //     }
        //     Turn::SlightRight => {
        //         // 0.5 second penalty for slight right
        //         Time::new(0.5)
        //     }
        //     Turn::Right => {
        //         // 1 second penalty for right
        //         Time::new(1.0)
        //     }
        //     Turn::SharpRight => {
        //         // 1.5 second penalty for sharp right
        //         Time::new(1.5)
        //     }
        //     Turn::SlightLeft => {
        //         // 1 second penalty for slight left
        //         Time::new(1.0)
        //     }
        //     Turn::Left => {
        //         // 2.5 second penalty for left
        //         Time::new(2.5)
        //     }
        //     Turn::SharpLeft => {
        //         // 3.5 second penalty for sharp left
        //         Time::new(3.5)
        //     }
        //     Turn::UTurn => {
        //         // 9.5 second penalty for U-turn
        //         Time::new(9.5)
        //     }
        // };
        // let time =
        //     TimeUnit::Seconds.convert(time_cost, &self.energy_model_service.time_unit);
        // let time_idx = self.time_model.get_state_variable(&"time", state)?;
        // let mut updated_state = state.clone();
        // Ok(Some(updated_state))
        // }
        // }
    }

    fn estimate_traversal(
        &self,
        src: &Vertex,
        dst: &Vertex,
        state: &[StateVar],
    ) -> Result<TraversalState, TraversalModelError> {
        let distance = haversine::coord_distance(
            &src.coordinate,
            &dst.coordinate,
            self.energy_model_service.distance_unit,
        )
        .map_err(TraversalModelError::NumericError)?;

        if distance == Distance::ZERO {
            return Ok(state.to_vec());
        }

        // let time_state = &state[0..self.vehicle_state_index];
        // let vehicle_state = &state[self.vehicle_state_index..];
        let mut updated_state = self.time_model.estimate_traversal(src, dst, state)?;
        let best_case_result = self
            .vehicle
            .best_case_energy_state((distance, self.energy_model_service.distance_unit), state)?;

        updated_state.extend(best_case_result.updated_state);
        Ok(updated_state)
    }
}

impl EnergyTraversalModel {
    pub fn new(
        energy_model_service: Arc<EnergyModelService>,
        conf: &serde_json::Value,
        state_model: Arc<StateModel>,
    ) -> Result<EnergyTraversalModel, TraversalModelError> {
        let time_model = energy_model_service
            .time_model_service
            .build(conf, state_model.clone())?;
        // let vehicle_state_index = time_model.initial_state().len();

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

        let vehicle = match energy_model_service
            .vehicle_library
            .get(&prediction_model_name)
        {
            None => {
                let model_names: Vec<&String> =
                    energy_model_service.vehicle_library.keys().collect();
                Err(TraversalModelError::BuildError(format!(
                    "No vehicle found with model_name = '{}', try one of: {:?}",
                    prediction_model_name, model_names
                )))
            }
            Some(mr) => Ok(mr.clone()),
        }?
        .update_from_query(conf)?;

        // let mut state_variable_names = time_model.state_variable_names();
        // state_variable_names.extend(vehicle.state_variable_names());
        // let state_variables = state_variable_names
        //     .into_iter()
        //     .enumerate()
        //     .map(|(idx, name)| (name, idx))
        //     .collect::<HashMap<_, _>>();
        Ok(EnergyTraversalModel {
            energy_model_service,
            time_model,
            vehicle,
            // vehicle_state_index,
            // state_variables,
            state_model,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::routee::{
        prediction::load_prediction_model, prediction::model_type::ModelType,
        vehicle::default::ice::ICE,
    };

    use super::*;
    use geo::coord;
    use routee_compass_core::{
        model::{
            property::{edge::Edge, vertex::Vertex},
            road_network::{edge_id::EdgeId, vertex_id::VertexId},
            traversal::default::{
                speed_traversal_engine::SpeedTraversalEngine,
                speed_traversal_model::SpeedTraversalModel,
                speed_traversal_service::SpeedLookupService,
            },
        },
        util::geo::coord::InternalCoord,
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
        let heading_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("routee")
            .join("test")
            .join("headings.csv");
        let model_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("routee")
            .join("test")
            .join("Toyota_Camry.bin");
        let v = Vertex {
            vertex_id: VertexId(0),
            coordinate: InternalCoord(coord! {x: -86.67, y: 36.12}),
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

        let state_model = Arc::new(StateModel::new(json!({
            "distance": ""
        }))?);
        let camry = ICE::new("Toyota_Camry".to_string(), model_record).unwrap();

        let mut model_library: HashMap<String, Arc<dyn VehicleType>> = HashMap::new();
        model_library.insert("Toyota_Camry".to_string(), Arc::new(camry));

        let time_engine =
            SpeedTraversalEngine::new(&speed_file_path, SpeedUnit::KilometersPerHour, None, None)
                .unwrap();
        let time_service = SpeedLookupService {
            e: Arc::new(time_model),
        };

        let service = EnergyModelService::new(
            Arc::new(time_service),
            SpeedUnit::MilesPerHour,
            // &speed_file_path,
            &Some(grade_file_path),
            // SpeedUnit::KilometersPerHour,
            Some(GradeUnit::Millis),
            None,
            None,
            model_library,
            &Some(heading_file_path),
        )
        .unwrap();
        let arc_service = Arc::new(service);
        let conf = serde_json::json!({
            "model_name": "Toyota_Camry",
        });
        let model = EnergyTraversalModel::new(arc_service, &conf, state_model.clone()).unwrap();
        let initial = model.initial_state();
        let e1 = mock_edge(0);
        // 100 meters @ 10kph should take 36 seconds ((0.1/10) * 3600)
        let result = model.traverse_edge(&v, &e1, &v, &initial).unwrap();
        println!("{:?}", result);
    }
}
