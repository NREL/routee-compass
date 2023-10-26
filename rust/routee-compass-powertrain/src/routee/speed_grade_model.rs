use super::speed_grade_model_service::SpeedGradeModelService;
use routee_compass_core::model::cost::cost::Cost;
use routee_compass_core::model::graph::edge_id::EdgeId;
use routee_compass_core::model::property::edge::Edge;
use routee_compass_core::model::property::vertex::Vertex;
use routee_compass_core::model::traversal::default::speed_lookup_model::get_speed;
use routee_compass_core::model::traversal::state::state_variable::StateVar;
use routee_compass_core::model::traversal::state::traversal_state::TraversalState;
use routee_compass_core::model::traversal::traversal_model::TraversalModel;
use routee_compass_core::model::traversal::traversal_model_error::TraversalModelError;
use routee_compass_core::model::traversal::traversal_result::TraversalResult;
use routee_compass_core::util::geo::haversine;
use routee_compass_core::util::unit::as_f64::AsF64;
use routee_compass_core::util::unit::*;
use std::sync::Arc;

const ZERO_ENERGY: f64 = 1e-9;

pub struct SpeedGradeModel {
    pub service: Arc<SpeedGradeModelService>,
    pub energy_cost_coefficient: f64,
}

impl TraversalModel for SpeedGradeModel {
    fn initial_state(&self) -> TraversalState {
        // distance, time, energy
        vec![StateVar(0.0), StateVar(0.0), StateVar(0.0)]
    }
    /// estimate the cost of traveling between two vertices.
    /// given a distance estimate,
    fn cost_estimate(
        &self,
        src: &Vertex,
        dst: &Vertex,
        _state: &TraversalState,
    ) -> Result<Cost, TraversalModelError> {
        let distance = haversine::coord_distance(
            src.coordinate,
            dst.coordinate,
            self.service.output_distance_unit.clone(),
        )
        .map_err(TraversalModelError::NumericError)?;

        if distance == Distance::ZERO {
            return Ok(Cost::ZERO);
        }

        let (energy, _energy_unit) = Energy::create(
            self.service.ideal_energy_rate,
            self.service.energy_model_energy_rate_unit.clone(),
            distance,
            self.service.output_distance_unit.clone(),
        )?;

        let time: Time = Time::create(
            self.service.max_speed.clone(),
            self.service.speeds_table_speed_unit.clone(),
            distance,
            self.service.output_distance_unit.clone(),
            self.service.output_time_unit.clone(),
        )?;

        let total_cost = create_cost(energy, time, self.energy_cost_coefficient);
        Ok(total_cost)
    }

    fn traversal_cost(
        &self,
        _src: &Vertex,
        edge: &Edge,
        _dst: &Vertex,
        state: &TraversalState,
    ) -> Result<TraversalResult, TraversalModelError> {
        let distance = BASE_DISTANCE_UNIT.convert(edge.distance, self.service.output_distance_unit);
        let speed = get_speed(&self.service.speed_table, edge.edge_id)?;
        let grade = get_grade(&self.service.grade_table, edge.edge_id)?;

        let time: Time = Time::create(
            speed,
            self.service.speeds_table_speed_unit.clone(),
            distance,
            self.service.output_distance_unit.clone(),
            self.service.output_time_unit.clone(),
        )?;

        let (energy_rate, _energy_rate_unit) = self.service.energy_model.predict(
            speed,
            self.service.speeds_table_speed_unit,
            grade,
            self.service.grade_table_grade_unit,
        )?;

        let energy_rate_real_world = energy_rate * self.service.real_world_energy_adjustment;

        let (mut energy, _energy_unit) = Energy::create(
            energy_rate_real_world,
            self.service.energy_model_energy_rate_unit.clone(),
            distance,
            self.service.output_distance_unit.clone(),
        )?;

        if energy.as_f64() < 0.0 {
            energy = Energy::new(ZERO_ENERGY);
            log::debug!("negative energy encountered, setting to 1e-9");
        }

        let total_cost = create_cost(energy, time, self.energy_cost_coefficient);
        let updated_state = update_state(&state, distance, time, energy);
        let result = TraversalResult {
            total_cost,
            updated_state,
        };
        Ok(result)
    }

    fn serialize_state(&self, state: &TraversalState) -> serde_json::Value {
        let distance = get_distance_from_state(state);
        let time = get_time_from_state(state);
        let energy = get_energy_from_state(state);
        serde_json::json!({
            "distance": distance,
            "time": time,
            "energy": energy,
        })
    }

    fn serialize_state_info(&self, _state: &TraversalState) -> serde_json::Value {
        let energy_unit = self
            .service
            .energy_model_energy_rate_unit
            .associated_energy_unit();
        serde_json::json!({
            "distance_unit": self.service.output_distance_unit,
            "time_unit": self.service.output_time_unit,
            "energy_unit": energy_unit,
        })
    }
}

impl TryFrom<(Arc<SpeedGradeModelService>, &serde_json::Value)> for SpeedGradeModel {
    type Error = TraversalModelError;

    fn try_from(
        input: (Arc<SpeedGradeModelService>, &serde_json::Value),
    ) -> Result<Self, Self::Error> {
        let (service, conf) = input;

        match conf.get(String::from("energy_cost_coefficient")) {
            None => {
                log::debug!("no energy_cost_coefficient provided");
                Ok(SpeedGradeModel {
                    service: service.clone(),
                    energy_cost_coefficient: 1.0,
                })
            }
            Some(v) => {
                let f = v.as_f64().ok_or(TraversalModelError::BuildError(format!(
                    "expected 'energy_cost_coefficient' value to be numeric, found {}",
                    v
                )))?;
                if f < 0.0 || 1.0 < f {
                    Err(TraversalModelError::BuildError(format!("expected 'energy_cost_coefficient' value to be numeric in range [0.0, 1.0], found {}", f)))
                } else {
                    log::debug!("using energy_cost_coefficient of {}", f);
                    Ok(SpeedGradeModel {
                        service: service.clone(),
                        energy_cost_coefficient: f,
                    })
                }
            }
        }
    }
}

fn create_cost(energy: Energy, time: Time, energy_percent: f64) -> Cost {
    let energy_scaled = energy * energy_percent;
    let energy_cost = Cost::from(energy_scaled);
    let time_scaled = time * (1.0 - energy_percent);
    let time_cost = Cost::from(time_scaled);
    let total_cost = energy_cost + time_cost;
    total_cost
}

fn update_state(
    state: &TraversalState,
    distance: Distance,
    time: Time,
    energy: Energy,
) -> TraversalState {
    let mut updated_state = state.clone();
    updated_state[0] = state[0] + distance.into();
    updated_state[1] = state[1] + time.into();
    updated_state[2] = state[2] + energy.into();
    return updated_state;
}

fn get_distance_from_state(state: &TraversalState) -> Distance {
    return Distance::new(state[0].0);
}

fn get_time_from_state(state: &TraversalState) -> Time {
    return Time::new(state[1].0);
}

fn get_energy_from_state(state: &TraversalState) -> Energy {
    return Energy::new(state[2].0);
}

/// look up the grade from the grade table
pub fn get_grade(
    grade_table: &Option<Vec<Grade>>,
    edge_id: EdgeId,
) -> Result<Grade, TraversalModelError> {
    match grade_table {
        None => Ok(Grade::ZERO),
        Some(gt) => {
            let grade: &Grade = gt.get(edge_id.as_usize()).ok_or(
                TraversalModelError::MissingIdInTabularCostFunction(
                    format!("{}", edge_id),
                    String::from("EdgeId"),
                    String::from("grade table"),
                ),
            )?;
            Ok(*grade)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::routee::model_type::ModelType;

    use super::*;
    use geo::coord;
    use routee_compass_core::model::{
        graph::{edge_id::EdgeId, vertex_id::VertexId},
        property::{edge::Edge, vertex::Vertex},
    };
    use std::path::PathBuf;

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
        let speed_file_name = speed_file_path.to_str().unwrap();
        let grade_file_name = grade_file_path.to_str().unwrap();
        let model_file_name = model_file_path.to_str().unwrap();
        let v = Vertex {
            vertex_id: VertexId(0),
            coordinate: coord! {x: -86.67, y: 36.12},
        };
        fn mock_edge(edge_id: usize) -> Edge {
            return Edge {
                edge_id: EdgeId(edge_id),
                src_vertex_id: VertexId(0),
                dst_vertex_id: VertexId(1),
                distance: Distance::new(100.0),
            };
        }
        let speed_file = String::from(speed_file_name);
        let grade_file = String::from(grade_file_name);
        let routee_model_path = String::from(model_file_name);
        let service = SpeedGradeModelService::new(
            speed_file,
            SpeedUnit::KilometersPerHour,
            None,
            None,
            routee_model_path,
            ModelType::Smartcore,
            None,
            SpeedUnit::MilesPerHour,
            GradeUnit::Millis,
            EnergyRateUnit::GallonsGasolinePerMile,
            None,
            None,
            None,
        )
        .unwrap();
        let arc_service = Arc::new(service);
        let conf = serde_json::json!({});
        let model = SpeedGradeModel::try_from((arc_service, &conf)).unwrap();
        let initial = model.initial_state();
        let e1 = mock_edge(0);
        // 100 meters @ 10kph should take 36 seconds ((0.1/10) * 3600)
        let result = model.traversal_cost(&v, &e1, &v, &initial).unwrap();
        println!("{}, {:?}", result.total_cost, result.updated_state);
    }
}
