use super::speed_grade_model_service::SpeedGradeModelService;
use compass_core::model::cost::cost::Cost;
use compass_core::model::property::edge::Edge;
use compass_core::model::property::vertex::Vertex;
use compass_core::model::traversal::state::state_variable::StateVar;
use compass_core::model::traversal::state::traversal_state::TraversalState;
use compass_core::model::traversal::traversal_model::TraversalModel;
use compass_core::model::traversal::traversal_model_error::TraversalModelError;
use compass_core::model::traversal::traversal_result::TraversalResult;
use compass_core::util::geo::haversine::coord_distance_km;
use compass_core::util::unit::*;
use std::sync::Arc;

pub struct SpeedGradeModel {
    pub service: Arc<SpeedGradeModelService>,
    pub energy_cost_coefficient: f64,
}

impl TraversalModel for SpeedGradeModel {
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
        let (energy, _energy_unit) = Energy::create(
            self.service.minimum_energy_rate,
            self.service.energy_model_energy_rate_unit.clone(),
            distance,
            DistanceUnit::Kilometers,
        )?;
        Ok(Cost::from(energy))
    }
    fn traversal_cost(
        &self,
        _src: &Vertex,
        edge: &Edge,
        _dst: &Vertex,
        state: &TraversalState,
    ) -> Result<TraversalResult, TraversalModelError> {
        let time_unit = self.service.speeds_table_speed_unit.associated_time_unit();

        let speed = self
            .service
            .speed_table
            .get(edge.edge_id.as_usize())
            .ok_or(TraversalModelError::MissingIdInTabularCostFunction(
                format!("{}", edge.edge_id),
                String::from("EdgeId"),
                String::from("speed table"),
            ))?;

        let time: Time = Time::create(
            *speed,
            self.service.speeds_table_speed_unit.clone(),
            edge.distance,
            DistanceUnit::Meters,
            time_unit.clone(),
        )?;
        let grade = edge.grade;
        let (energy_rate, _energy_rate_unit) = self.service.energy_model.predict(
            *speed,
            self.service.speeds_table_speed_unit,
            grade,
            self.service.graph_grade_unit,
        )?;

        let energy_rate_safe = if energy_rate < self.service.minimum_energy_rate {
            self.service.minimum_energy_rate
        } else {
            energy_rate
        };
        let (energy, _energy_unit) = Energy::create(
            energy_rate_safe,
            self.service.energy_model_energy_rate_unit.clone(),
            edge.distance,
            DistanceUnit::Meters,
        )?;

        let total_cost = create_cost(energy, time, self.energy_cost_coefficient);
        let updated_state = update_state(&state, time, energy);
        let result = TraversalResult {
            total_cost,
            updated_state,
        };
        Ok(result)
    }

    fn summary(&self, state: &TraversalState) -> serde_json::Value {
        let data_time_unit = self.service.speeds_table_speed_unit.associated_time_unit();
        let time = get_time_from_state(state);
        let time_output = data_time_unit.convert(time, self.service.output_time_unit.clone());
        let energy = get_energy_from_state(state);
        let energy_unit = self
            .service
            .energy_model_energy_rate_unit
            .associated_energy_unit();
        serde_json::json!({
            "energy": energy,
            "energy_unit": energy_unit,
            "time": time_output,
            "time_unit": self.service.output_time_unit.clone()
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

#[cfg(test)]
mod tests {
    use crate::routee::model_type::ModelType;

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
                grade: Grade::ZERO,
            };
        }
        let speed_file = String::from(speed_file_name);
        let routee_model_path = String::from(model_file_name);
        let service = SpeedGradeModelService::new(
            speed_file,
            SpeedUnit::KilometersPerHour,
            routee_model_path,
            ModelType::Smartcore,
            SpeedUnit::MilesPerHour,
            GradeUnit::Decimal,
            GradeUnit::Millis,
            EnergyRateUnit::GallonsGasolinePerMile,
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
