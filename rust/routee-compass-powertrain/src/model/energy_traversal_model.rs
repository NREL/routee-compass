use super::energy_model_ops::get_grade;
use super::energy_model_service::EnergyModelService;
use super::vehicle::VehicleType;
use routee_compass_core::model::network::{Edge, Vertex};
use routee_compass_core::model::state::StateFeature;
use routee_compass_core::model::state::StateModel;
use routee_compass_core::model::state::StateVariable;
use routee_compass_core::model::traversal::TraversalModel;
use routee_compass_core::model::traversal::TraversalModelError;
use routee_compass_core::model::unit::*;
use routee_compass_core::util::geo::haversine;
use std::borrow::Cow;
use std::sync::Arc;

pub struct EnergyTraversalModel {
    pub energy_model_service: Arc<EnergyModelService>,
    pub time_model: Arc<dyn TraversalModel>,
    pub vehicle: Arc<dyn VehicleType>,
}

impl TraversalModel for EnergyTraversalModel {
    /// inject the state features required by the VehicleType
    fn state_features(&self) -> Vec<(String, StateFeature)> {
        let mut features = self.vehicle.state_features();
        features.extend(self.time_model.state_features());
        features
    }

    fn traverse_edge(
        &self,
        trajectory: (&Vertex, &Edge, &Vertex),
        state: &mut Vec<StateVariable>,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        let (_, edge, _) = trajectory;

        // perform time traversal
        let prev = state.to_vec();
        self.time_model
            .traverse_edge(trajectory, state, state_model)?;

        // calculate time delta
        let prev_time = state_model.get_time(
            &prev,
            &Self::TIME.into(),
            &self
                .energy_model_service
                .time_model_speed_unit
                .associated_time_unit(),
        )?;
        let current_time = state_model.get_time(
            state,
            &Self::TIME.into(),
            &self
                .energy_model_service
                .time_model_speed_unit
                .associated_time_unit(),
        )?;
        let time_delta = current_time - prev_time;

        // perform vehicle energy traversal
        let grade = get_grade(&self.energy_model_service.grade_table, edge.edge_id)?;

        let mut distance_in_energy_model_unit = Cow::Borrowed(&edge.distance);
        baseunit::DISTANCE_UNIT.convert(
            &mut distance_in_energy_model_unit,
            &self
                .energy_model_service
                .time_model_speed_unit
                .associated_distance_unit(),
        )?;
        let speed_tuple = Speed::from_distance_and_time(
            (
                &distance_in_energy_model_unit,
                &self
                    .energy_model_service
                    .time_model_speed_unit
                    .associated_distance_unit(),
            ),
            (
                &time_delta,
                &self
                    .energy_model_service
                    .time_model_speed_unit
                    .associated_time_unit(),
            ),
        )?;
        self.vehicle.consume_energy(
            speed_tuple,
            (grade, self.energy_model_service.grade_table_grade_unit),
            (edge.distance, baseunit::DISTANCE_UNIT),
            state,
            state_model,
        )?;

        Ok(())
    }

    fn estimate_traversal(
        &self,
        od: (&Vertex, &Vertex),
        state: &mut Vec<StateVariable>,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        let (src, dst) = od;
        let distance = haversine::coord_distance(
            &src.coordinate,
            &dst.coordinate,
            self.energy_model_service.distance_unit,
        )
        .map_err(|e| {
            TraversalModelError::TraversalModelFailure(format!(
                "could not compute haversine distance between {} and {}: {}",
                src, dst, e
            ))
        })?;

        if distance == Distance::ZERO {
            return Ok(());
        }

        self.time_model.estimate_traversal(od, state, state_model)?;
        self.vehicle.best_case_energy_state(
            (distance, self.energy_model_service.distance_unit),
            state,
            state_model,
        )?;

        Ok(())
    }
}

impl EnergyTraversalModel {
    const TIME: &'static str = "time";

    pub fn new(
        energy_model_service: Arc<EnergyModelService>,
        conf: &serde_json::Value,
    ) -> Result<EnergyTraversalModel, TraversalModelError> {
        let time_model = energy_model_service.time_model_service.build(conf)?;

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

        let vehicle_lookup = energy_model_service
            .vehicle_library
            .get(&prediction_model_name);
        let vehicle_initial = vehicle_lookup.cloned().ok_or_else(|| {
            let model_names: Vec<&String> = energy_model_service.vehicle_library.keys().collect();
            TraversalModelError::BuildError(format!(
                "No vehicle found with model_name = '{}', try one of: {:?}",
                prediction_model_name, model_names
            ))
        })?;
        // allow user to customize this vehicle instance if applicable
        let vehicle = vehicle_initial.update_from_query(conf)?;

        Ok(EnergyTraversalModel {
            energy_model_service,
            time_model,
            vehicle,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        prediction::load_prediction_model, prediction::ModelType, vehicle::default::ICE,
    };
    use geo::coord;
    use routee_compass_core::{
        model::{
            network::{Edge, EdgeId, Vertex, VertexId},
            traversal::default::{SpeedLookupService, SpeedTraversalEngine},
        },
        util::geo::InternalCoord,
    };
    use std::{collections::HashMap, path::PathBuf};

    #[test]
    fn test_edge_cost_lookup_from_file() {
        let speed_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("model")
            .join("test")
            .join("velocities.txt");
        let grade_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("model")
            .join("test")
            .join("grades.txt");
        let model_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("model")
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
                distance: Distance::from(100.0),
            }
        }
        let model_record = load_prediction_model(
            "Toyota_Camry".to_string(),
            &model_file_path,
            ModelType::Smartcore,
            SpeedUnit::MPH,
            GradeUnit::Decimal,
            EnergyRateUnit::GGPM,
            None,
            None,
            None,
        )
        .unwrap();

        let state_model = Arc::new(StateModel::empty());
        let camry = ICE::new("Toyota_Camry".to_string(), model_record).unwrap();

        let mut model_library: HashMap<String, Arc<dyn VehicleType>> = HashMap::new();
        model_library.insert("Toyota_Camry".to_string(), Arc::new(camry));

        let time_engine = Arc::new(
            SpeedTraversalEngine::new(&speed_file_path, SpeedUnit::KPH, None, None).unwrap(),
        );
        let time_service = SpeedLookupService { e: time_engine };

        let service = EnergyModelService::new(
            Arc::new(time_service),
            SpeedUnit::MPH,
            // &speed_file_path,
            &Some(grade_file_path),
            // SpeedUnit::KPH,
            GradeUnit::Millis,
            None,
            None,
            model_library,
        )
        .unwrap();
        let arc_service = Arc::new(service);
        let conf = serde_json::json!({
            "model_name": "Toyota_Camry",
        });
        let model = EnergyTraversalModel::new(arc_service, &conf).unwrap();
        let updated_state_model = state_model.extend(model.state_features()).unwrap();
        println!("{:?}", updated_state_model.to_vec());
        let mut state = updated_state_model.initial_state().unwrap();
        let e1 = mock_edge(0);
        // 100 meters @ 10kph should take 36 seconds ((0.1/10) * 3600)
        model
            .traverse_edge((&v, &e1, &v), &mut state, &updated_state_model)
            .unwrap();
        println!("{:?}", state);
    }
}
