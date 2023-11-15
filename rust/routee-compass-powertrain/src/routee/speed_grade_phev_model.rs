use crate::routee::speed_grade_model_ops::ZERO_ENERGY;

use super::prediction_model::SpeedGradePredictionModelRecord;
use super::speed_grade_model_ops::get_grade;
use super::speed_grade_model_service::SpeedGradeModelService;
use routee_compass_core::model::cost::Cost;
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

pub struct SpeedGradePHEVModel {
    pub service: Arc<SpeedGradeModelService>,
    pub charge_sustain_model_record: Arc<SpeedGradePredictionModelRecord>,
    pub charge_deplete_model_record: Arc<SpeedGradePredictionModelRecord>,
    pub energy_cost_coefficient: f64,
    pub starting_soc: f64,
}

impl TraversalModel for SpeedGradePHEVModel {
    fn initial_state(&self) -> TraversalState {
        vec![
            StateVar(0.0), // accumulated distance
            StateVar(0.0), // accumulated time
            StateVar(0.0), // accumulated electrical energy
            StateVar(0.0), // accumulated gasoline energy
            StateVar(0.0), // battery energy remaining
        ]
    }
    fn cost_estimate(
        &self,
        src: &Vertex,
        dst: &Vertex,
        _state: &TraversalState,
    ) -> Result<Cost, TraversalModelError> {
        let distance = haversine::coord_distance(
            src.coordinate,
            dst.coordinate,
            self.service.output_distance_unit,
        )
        .map_err(TraversalModelError::NumericError)?;

        if distance == Distance::ZERO {
            return Ok(Cost::ZERO);
        }

        // assume lowest energy cost scenario for a PHEV is to just use the battery
        let (electrical_energy, energy_unit) = Energy::create(
            self.charge_deplete_model_record.ideal_energy_rate,
            self.charge_deplete_model_record.energy_rate_unit,
            distance,
            self.service.output_distance_unit,
        )?;

        let time: Time = Time::create(
            self.service.max_speed,
            self.service.speeds_table_speed_unit,
            distance,
            self.service.output_distance_unit,
            self.service.output_time_unit.clone(),
        )?;

        let total_cost = create_cost(
            electrical_energy,
            energy_unit,
            Energy::new(0.0),
            self.charge_sustain_model_record
                .energy_rate_unit
                .associated_energy_unit(),
            time,
            self.energy_cost_coefficient,
        );
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
            self.service.speeds_table_speed_unit,
            distance,
            self.service.output_distance_unit,
            self.service.output_time_unit.clone(),
        )?;

        let battery_soc_percentage = get_battery_soc_percent(self, state)?;

        let (electrical_energy, electrical_energy_unit, gasoline_energy, gasoline_energy_unit) =
            get_phev_energy(self, battery_soc_percentage, speed, grade, distance)?;

        let total_cost = create_cost(
            electrical_energy,
            electrical_energy_unit,
            gasoline_energy,
            gasoline_energy_unit,
            time,
            self.energy_cost_coefficient,
        );
        let updated_state = update_state(state, distance, time, electrical_energy, gasoline_energy);
        let result = TraversalResult {
            total_cost,
            updated_state,
        };
        Ok(result)
    }

    fn serialize_state(&self, state: &TraversalState) -> serde_json::Value {
        let distance = get_distance_from_state(state);
        let time = get_time_from_state(state);
        let electrical_energy = get_electrical_energy_from_state(state);
        let gasoline_energy = get_gasoline_energy_from_state(state);
        let battery_soc_percent = get_battery_soc_percent(self, state).unwrap_or(0.0);
        serde_json::json!({
            "distance": distance,
            "time": time,
            "electrical_energy": electrical_energy,
            "gasoline_energy": gasoline_energy,
            "final_battery_soc": battery_soc_percent,
        })
    }

    fn serialize_state_info(&self, _state: &TraversalState) -> serde_json::Value {
        let electrical_energy_unit = self.charge_deplete_model_record.energy_rate_unit.associated_energy_unit();
        let gasoline_energy_unit = self.charge_sustain_model_record.energy_rate_unit.associated_energy_unit();
        serde_json::json!({
            "distance_unit": self.service.output_distance_unit,
            "time_unit": self.service.output_time_unit,
            "electrical_energy_unit": electrical_energy_unit,
            "gasoline_energy_unit": gasoline_energy_unit,
        })
    }
}

impl TryFrom<(Arc<SpeedGradeModelService>, &serde_json::Value)> for SpeedGradeModel {
    type Error = TraversalModelError;

    fn try_from(
        input: (Arc<SpeedGradeModelService>, &serde_json::Value),
    ) -> Result<Self, Self::Error> {
        let (service, conf) = input;

        let energy_cost_coefficient = match conf.get(String::from("energy_cost_coefficient")) {
            None => {
                log::debug!("no energy_cost_coefficient provided");
                1.0
            }
            Some(v) => {
                let f = v.as_f64().ok_or(TraversalModelError::BuildError(format!(
                    "expected 'energy_cost_coefficient' value to be numeric, found {}",
                    v
                )))?;
                if !(0.0..=1.0).contains(&f) {
                    return Err(TraversalModelError::BuildError(format!("expected 'energy_cost_coefficient' value to be numeric in range [0.0, 1.0], found {}", f)));
                } else {
                    log::debug!("using energy_cost_coefficient of {}", f);
                    f
                }
            }
        };

        let prediction_model_name = conf
            .get("model_name".to_string())
            .ok_or(TraversalModelError::BuildError(
                "No 'model_name' key provided in query".to_string(),
            ))?
            .as_str()
            .ok_or(TraversalModelError::BuildError(
                "Expected 'model_name' value to be string".to_string(),
            ))?
            .to_string();

        let model_record = match service.energy_model_library.get(&prediction_model_name) {
            None => {
                let model_names: Vec<&String> = service.energy_model_library.keys().collect();
                return Err(TraversalModelError::BuildError(format!(
                    "No energy model found with model_name = '{}', try one of: {:?}",
                    prediction_model_name, model_names
                )));
            }
            Some(mr) => mr.clone(),
        };

        Ok(SpeedGradeModel {
            service,
            model_record,
            energy_cost_coefficient,
        })
    }
}

fn create_cost(
    electrical_energy: Energy,
    electrical_energy_unit: EnergyUnit,
    gasoline_energy: Energy,
    gasoline_energy_unit: EnergyUnit,
    time: Time,
    energy_percent: f64,
) -> Cost {
    let electrical_energy_kwh =
        electrical_energy_unit.convert(electrical_energy, EnergyUnit::KilowattHours);
    let gasoline_energy_kwh =
        gasoline_energy_unit.convert(gasoline_energy, EnergyUnit::KilowattHours);
    let total_energy_kwh = electrical_energy_kwh + gasoline_energy_kwh;
    let energy_scaled = total_energy_kwh * energy_percent;
    let energy_cost = Cost::from(energy_scaled);
    let time_scaled = time * (1.0 - energy_percent);
    let time_cost = Cost::from(time_scaled);

    energy_cost + time_cost
}

fn update_state(
    state: &TraversalState,
    distance: Distance,
    time: Time,
    electrical_energy: Energy,
    gasoline_energy: Energy,
) -> TraversalState {
    let current_battery_energy = get_remaining_battery_energy_from_state(state);
    let new_battery_energy = current_battery_energy - electrical_energy;
    let mut updated_state = state.clone();
    updated_state[0] = state[0] + distance.into();
    updated_state[1] = state[1] + time.into();
    updated_state[2] = state[2] + electrical_energy.into();
    updated_state[3] = state[3] + gasoline_energy.into();
    updated_state[4] = state[4] + new_battery_energy.into();
    updated_state
}

fn get_distance_from_state(state: &TraversalState) -> Distance {
    Distance::new(state[0].0)
}

fn get_time_from_state(state: &TraversalState) -> Time {
    Time::new(state[1].0)
}

fn get_electrical_energy_from_state(state: &TraversalState) -> Energy {
    Energy::new(state[2].0)
}

fn get_gasoline_energy_from_state(state: &TraversalState) -> Energy {
    Energy::new(state[3].0)
}

fn get_remaining_battery_energy_from_state(state: &TraversalState) -> Energy {
    Energy::new(state[4].0)
}

fn get_battery_soc_percent(
    model: &SpeedGradePHEVModel,
    state: &TraversalState,
) -> Result<f64, TraversalModelError> {
    let battery_capacity = model.charge_deplete_model_record.battery_capacity.ok_or(
        TraversalModelError::InternalError("battery capacity not set on PHEV model".to_string()),
    )?;
    let battery_capacity_unit = model
        .charge_deplete_model_record
        .battery_capacity_unit
        .ok_or(TraversalModelError::InternalError(
            "battery capacity unit not set on PHEV model".to_string(),
        ))?;
    let battery_capacity_kwh =
        battery_capacity_unit.convert(battery_capacity, EnergyUnit::KilowattHours);

    let remaining_battery_energy = get_remaining_battery_energy_from_state(state);
    let remaining_battery_energy_unit = model
        .charge_deplete_model_record
        .energy_rate_unit
        .associated_energy_unit();

    let remaining_battery_energy_kwh =
        remaining_battery_energy_unit.convert(remaining_battery_energy, EnergyUnit::KilowattHours);

    let battery_soc_percent =
        (remaining_battery_energy_kwh.as_f64() / battery_capacity_kwh.as_f64()) * 100;
    Ok(battery_soc_percent)
}

/// Compute the energy for the PHEV by converting gasoline to kWh.
/// This uses a simplified operation in which we assume that if the battery
/// SOC is greater than zero we can just operate on battery to traverse a link.
/// This is not entirely realistic as it's possible to arrive at a link with
/// 0.001% SOC and still need to use gasoline to traverse the link.
///
/// In the future we could make this more sophisticated by calculating
/// the energy required to traverse the link using the battery and then
/// finding the point at which we would have to switch to gasoline
///
/// Returns a tuple of (electrical_energy, electrical_energy_unit, gasoline_energy, gasoline_energy_unit)
fn get_phev_energy(
    model: &SpeedGradePHEVModel,
    battery_soc_percent: f64,
    speed: Speed,
    grade: Grade,
    distance: Distance,
) -> Result<(Energy, EnergyUnit, Energy, EnergyUnit), TraversalModelError> {
    let electrical_energy_unit = model
        .charge_deplete_model_record
        .energy_rate_unit
        .associated_energy_unit();
    let gasoline_energy_unit = model
        .charge_sustain_model_record
        .energy_rate_unit
        .associated_energy_unit();

    if battery_soc_percent > 0.0 {
        // assume we can just use the battery
        let (pred_energy_rate, pred_energy_rate_unit) =
            model.charge_deplete_model_record.prediction_model.predict(
                speed,
                model.service.speeds_table_speed_unit,
                grade,
                model.service.grade_table_grade_unit,
            )?;
        let pred_energy_rate = pred_energy_rate
            * model
                .charge_deplete_model_record
                .real_world_energy_adjustment;
        let (mut pred_energy, pred_energy_unit) = Energy::create(
            pred_energy_rate,
            pred_energy_rate_unit,
            distance,
            model.service.output_distance_unit,
        )?;
        if pred_energy.as_f64() < 0.0 {
            pred_energy = Energy::new(ZERO_ENERGY);
            log::debug!("negative energy encountered, setting to 1e-9");
        }
        return Ok((
            pred_energy,
            pred_energy_unit,
            Energy::new(0.0),
            gasoline_energy_unit,
        ));
    } else {
        // just use the gasoline engine
        let (pred_energy_rate, pred_energy_rate_unit) =
            model.charge_sustain_model_record.prediction_model.predict(
                speed,
                model.service.speeds_table_speed_unit,
                grade,
                model.service.grade_table_grade_unit,
            )?;
        let pred_energy_rate = pred_energy_rate
            * model
                .charge_deplete_model_record
                .real_world_energy_adjustment;
        let (mut pred_energy, pred_energy_unit) = Energy::create(
            pred_energy_rate,
            pred_energy_rate_unit,
            distance,
            model.service.output_distance_unit,
        )?;
        if pred_energy.as_f64() < 0.0 {
            pred_energy = Energy::new(ZERO_ENERGY);
            log::debug!("negative energy encountered, setting to 1e-9");
        }
        return Ok((
            Energy::new(0.0),
            electrical_energy_unit,
            pred_energy,
            pred_energy_unit,
        ));
    }
}

#[cfg(test)]
mod tests {
    use crate::routee::model_type::ModelType;

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
        let model_record = SpeedGradePredictionModelRecord::new(
            "Toyota_Camry".to_string(),
            &model_file_path,
            ModelType::Smartcore,
            SpeedUnit::MilesPerHour,
            GradeUnit::Decimal,
            EnergyRateUnit::GallonsGasolinePerMile,
            None,
            None,
        )
        .unwrap();
        let mut model_library = HashMap::new();
        model_library.insert("Toyota_Camry".to_string(), Arc::new(model_record));

        let service = SpeedGradeModelService::new(
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
            "energy_cost_coefficient": 0.5,
        });
        let model = SpeedGradeModel::try_from((arc_service, &conf)).unwrap();
        let initial = model.initial_state();
        let e1 = mock_edge(0);
        // 100 meters @ 10kph should take 36 seconds ((0.1/10) * 3600)
        let result = model.traversal_cost(&v, &e1, &v, &initial).unwrap();
        println!("{}, {:?}", result.total_cost, result.updated_state);
    }
}
