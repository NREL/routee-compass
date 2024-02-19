use super::energy_model_ops::{get_grade, get_headings};
use super::energy_model_service::EnergyModelService;
use super::vehicle::vehicle_type::{VehicleState, VehicleType};
use routee_compass_core::model::property::edge::Edge;
use routee_compass_core::model::property::vertex::Vertex;
use routee_compass_core::model::road_network::turn::Turn;
use routee_compass_core::model::traversal::state::state_variable::StateVar;
use routee_compass_core::model::traversal::state::traversal_state::TraversalState;
use routee_compass_core::model::traversal::traversal_model::TraversalModel;
use routee_compass_core::model::traversal::traversal_model_error::TraversalModelError;
use routee_compass_core::model::traversal::traversal_model_service::TraversalModelService;
use routee_compass_core::model::unit::*;
use routee_compass_core::util::geo::haversine;
use std::collections::HashMap;
use std::sync::Arc;

pub struct EnergyTraversalModel {
    pub energy_model_service: Arc<EnergyModelService>,
    pub time_model: Arc<dyn TraversalModel>,
    pub vehicle: Arc<dyn VehicleType>,
    pub vehicle_state_index: usize,
    pub state_variables: HashMap<String, usize>,
}

impl TraversalModel for EnergyTraversalModel {
    fn initial_state(&self) -> TraversalState {
        // // distance, time
        // let mut initial_state = vec![StateVar(0.0), StateVar(0.0)];

        // // vehicle state gets slots 2..n
        // let vehicle_state = self.vehicle.initial_state();
        // initial_state.extend(vehicle_state);

        // initial_state

        let mut state = self.time_model.initial_state();
        state.extend(self.vehicle.initial_state());
        state
    }

    fn state_variable_names(&self) -> Vec<String> {
        self.state_variables
            .keys()
            .map(|k| k.clone())
            .collect::<Vec<_>>()
        // let mut dims = vec![String::from("distance"), String::from("time")];
        // dims.extend(self.vehicle.state_variable_names());
        // dims
    }

    fn get_state_variable(
        &self,
        key: &str,
        state: &[StateVar],
    ) -> Result<StateVar, TraversalModelError> {
        let index = self.state_variables.get(key).ok_or_else(|| {
            TraversalModelError::InternalError(format!("state variable {} not found in state", key))
        })?;
        let value_f64 = state.get(*index).ok_or_else(|| {
            TraversalModelError::InternalError(format!(
                "state variable index {} not found in state",
                index
            ))
        })?;
        Ok(*value_f64)
    }

    fn serialize_state(&self, state: &[StateVar]) -> serde_json::Value {
        let time_state = &state[0..self.vehicle_state_index];
        let vehicle_state = &state[self.vehicle_state_index..];
        let mut time_json = self.time_model.serialize_state(time_state);
        let energy_json = self.vehicle.serialize_state(vehicle_state);

        use serde_json::Value::Object;
        let result = match (time_json, energy_json) {
            (Object(ref mut a), Object(ref b)) => {
                for (k, v) in b.to_owned() {
                    a.insert(k, v);
                }
                serde_json::json!(a)
            }
            _ => {
                serde_json::json!({"internal error": "unable to serialize energy and time states as expected"})
            }
        };
        result

        // if let Some(obj) = self.time_model.serialize_state(state).as_object_mut() {
        //     let vehicle_state = &state[self.vehicle_state_index..];
        //     self.vehicle.vehicle_state(vehicle_state)
        //     for (k, v) in vehicle_json {

        //     }
        // }
        // let mut json = self.time_model.serialize_state(state);

        // // let mut obj = json.as_object_mut();
        // let distance = get_distance_from_state(state);
        // let time = get_time_from_state(state);
        // let vehicle_state = get_vehicle_state_from_state(state);
        // let vehicle_state_summary = self.vehicle.serialize_state(vehicle_state);
        // serde_json::json!({
        //     "distance": distance,
        //     "time": time,
        //     "vehicle": vehicle_state_summary,
        // })
    }

    fn serialize_state_info(&self, state: &[StateVar]) -> serde_json::Value {
        let vehicle_state = get_vehicle_state_from_state(state);
        let vehicle_state_info = self.vehicle.serialize_state_info(vehicle_state);
        serde_json::json!({
            "distance_unit": self.energy_model_service.output_distance_unit,
            "time_unit": self.energy_model_service.output_time_unit,
            "vehicle_info": vehicle_state_info,
        })
    }

    fn traverse_edge(
        &self,
        src: &Vertex,
        edge: &Edge,
        dst: &Vertex,
        state: &[StateVar],
    ) -> Result<TraversalState, TraversalModelError> {
        let distance = BASE_DISTANCE_UNIT.convert(
            edge.distance,
            self.energy_model_service.output_distance_unit,
        );

        let time_state = &state[0..self.vehicle_state_index];
        let vehicle_state = &state[self.vehicle_state_index..];
        let time_state_02 = self.time_model.traverse_edge(src, edge, dst, time_state)?;

        // 1. grab the "time" value from the time state and calculate the delta from the previous time
        // DONE
        let time_next = self.time_model.get_state_variable("time", &time_state_02)?;
        let time_prev = self.get_state_variable("time", state)?;
        let time_delta: Time = Time::new(time_next.0 - time_prev.0);
        let speed = Speed::from((distance, time_delta));

        // 2. using the distance, compute the speed and speed unit
        // UH OH, we don't have access to the speed unit from here!
        // we don't have a clean way to find out :'(
        // let speed_unit = SpeedUnit::from((
        //     self.energy_model_service.output_distance_unit,
        //     self.time_model,
        // ));

        // 3. pass that into the energy model

        // let time_diff =

        // let speed = get_speed(&self.energy_model_service.speed_table, edge.edge_id)?;
        let grade = get_grade(&self.energy_model_service.grade_table, edge.edge_id)?;

        // let time: Time = Time::create(
        //     speed,
        //     self.energy_model_service.speeds_table_speed_unit,
        //     distance,
        //     self.energy_model_service.output_distance_unit,
        //     self.energy_model_service.output_time_unit.clone(),
        // )?;

        // let energy_result = self.vehicle.consume_energy(
        //     (speed, self.energy_model_service.speeds_table_speed_unit),
        //     (grade, self.energy_model_service.grade_table_grade_unit),
        //     (distance, self.energy_model_service.output_distance_unit),
        //     get_vehicle_state_from_state(state),
        // )?;

        // let updated_state = update_state(state, distance, time, energy_result.updated_state);
        let updated_state = vec![];
        Ok(updated_state)
    }

    fn access_edge(
        &self,
        _v1: &Vertex,
        src: &Edge,
        _v2: &Vertex,
        dst: &Edge,
        _v3: &Vertex,
        state: &[StateVar],
    ) -> Result<Option<TraversalState>, TraversalModelError> {
        match self.energy_model_service.headings_table.as_deref() {
            None => Ok(None),
            Some(headings_table) => {
                let src_heading = get_headings(headings_table, src.edge_id)?;
                let dst_heading = get_headings(headings_table, dst.edge_id)?;
                let angle = src_heading.next_edge_angle(&dst_heading);
                let turn = Turn::from_angle(angle)?;
                let time_cost = match turn {
                    Turn::NoTurn => {
                        // no penalty for straight
                        Time::new(0.0)
                    }
                    Turn::SlightRight => {
                        // 0.5 second penalty for slight right
                        Time::new(0.5)
                    }
                    Turn::Right => {
                        // 1 second penalty for right
                        Time::new(1.0)
                    }
                    Turn::SharpRight => {
                        // 1.5 second penalty for sharp right
                        Time::new(1.5)
                    }
                    Turn::SlightLeft => {
                        // 1 second penalty for slight left
                        Time::new(1.0)
                    }
                    Turn::Left => {
                        // 2.5 second penalty for left
                        Time::new(2.5)
                    }
                    Turn::SharpLeft => {
                        // 3.5 second penalty for sharp left
                        Time::new(3.5)
                    }
                    Turn::UTurn => {
                        // 9.5 second penalty for U-turn
                        Time::new(9.5)
                    }
                };
                let time = TimeUnit::Seconds
                    .convert(time_cost, &self.energy_model_service.output_time_unit);
                let updated_state = add_time_to_state(state, time);
                Ok(Some(updated_state))
            }
        }
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
            self.energy_model_service.output_distance_unit,
        )
        .map_err(TraversalModelError::NumericError)?;

        if distance == Distance::ZERO {
            return Ok(state.to_vec());
        }

        // 1. self.time_model.estimate_traversal
        let time_state = &state[0..self.vehicle_state_index];
        let vehicle_state = &state[self.vehicle_state_index..];
        let mut updated_state = self.time_model.estimate_traversal(src, dst, time_state)?;
        // let time: Time = Time::create(
        //     self.energy_model_service.max_speed,
        //     self.energy_model_service.speeds_table_speed_unit,
        //     distance,
        //     self.energy_model_service.output_distance_unit,
        //     self.energy_model_service.output_time_unit.clone(),
        // )?;

        let best_case_result = self.vehicle.best_case_energy_state(
            (distance, self.energy_model_service.output_distance_unit),
            vehicle_state,
        )?;

        updated_state.extend(best_case_result.updated_state);
        // let updated_state = update_state(state, distance, time, best_case_result.updated_state);
        Ok(updated_state)
    }
}

impl EnergyTraversalModel {
    pub fn new(
        energy_model_service: Arc<EnergyModelService>,
        time_model_service: Arc<dyn TraversalModelService>,
        conf: &serde_json::Value,
    ) -> Result<EnergyTraversalModel, TraversalModelError> {
        let time_model = time_model_service.build(conf)?;
        let vehicle_state_index = time_model.initial_state().len();

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

        let mut state_variable_names = time_model.state_variable_names();
        state_variable_names.extend(vehicle.state_variable_names());
        let state_variables = state_variable_names
            .into_iter()
            .enumerate()
            .map(|(idx, name)| (name, idx))
            .collect::<HashMap<_, _>>();
        Ok(EnergyTraversalModel {
            energy_model_service,
            time_model,
            vehicle,
            vehicle_state_index,
            state_variables,
        })
    }
}

fn update_state(
    state: &[StateVar],
    distance: Distance,
    time: Time,
    vehicle_state: VehicleState,
) -> TraversalState {
    let mut updated_state = Vec::with_capacity(state.len());

    updated_state.push(state[0] + distance.into());
    updated_state.push(state[1] + time.into());
    updated_state.extend(vehicle_state);
    updated_state
}

fn add_time_to_state(state: &[StateVar], time: Time) -> TraversalState {
    let mut updated_state = state.to_vec();
    updated_state[1] = state[1] + time.into();
    updated_state
}
fn get_distance_from_state(state: &[StateVar]) -> Distance {
    Distance::new(state[0].0)
}

fn get_time_from_state(state: &[StateVar]) -> Time {
    Time::new(state[1].0)
}

fn get_vehicle_state_from_state(state: &[StateVar]) -> &[StateVar] {
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
    use routee_compass_core::{
        model::{
            property::{edge::Edge, vertex::Vertex},
            road_network::{edge_id::EdgeId, vertex_id::VertexId},
            traversal::default::{
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

        let camry = ICE::new("Toyota_Camry".to_string(), model_record).unwrap();

        let mut model_library: HashMap<String, Arc<dyn VehicleType>> = HashMap::new();
        model_library.insert("Toyota_Camry".to_string(), Arc::new(camry));

        let time_model =
            SpeedTraversalModel::new(&speed_file_path, SpeedUnit::KilometersPerHour, None, None)
                .unwrap();
        let time_service = SpeedLookupService {
            m: Arc::new(time_model),
        };

        let service = EnergyModelService::new(
            time_service,
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
        let model = EnergyTraversalModel::try_from((arc_service, &conf)).unwrap();
        let initial = model.initial_state();
        let e1 = mock_edge(0);
        // 100 meters @ 10kph should take 36 seconds ((0.1/10) * 3600)
        let result = model.traverse_edge(&v, &e1, &v, &initial).unwrap();
        println!("{:?}", result);
    }
}
