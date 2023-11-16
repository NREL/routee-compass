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
    pub starting_battery_energy: Energy,
}

impl TraversalModel for SpeedGradePHEVModel {
    fn initial_state(&self) -> TraversalState {
        vec![
            StateVar(0.0),                                   // accumulated distance
            StateVar(0.0),                                   // accumulated time
            StateVar(0.0),                                   // accumulated electrical energy
            StateVar(0.0),                                   // accumulated gasoline energy
            StateVar(self.starting_battery_energy.as_f64()), // battery energy remaining
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
        let electrical_energy_unit = self
            .charge_deplete_model_record
            .energy_rate_unit
            .associated_energy_unit();
        let gasoline_energy_unit = self
            .charge_sustain_model_record
            .energy_rate_unit
            .associated_energy_unit();
        serde_json::json!({
            "distance_unit": self.service.output_distance_unit,
            "time_unit": self.service.output_time_unit,
            "electrical_energy_unit": electrical_energy_unit,
            "gasoline_energy_unit": gasoline_energy_unit,
        })
    }
}

impl TryFrom<(Arc<SpeedGradeModelService>, &serde_json::Value)> for SpeedGradePHEVModel {
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

        let model_name = conf
            .get("model_name".to_string())
            .ok_or(TraversalModelError::BuildError(
                "No 'model_name' key provided in query".to_string(),
            ))?
            .as_str()
            .ok_or(TraversalModelError::BuildError(
                "Expected 'model_name' value to be string".to_string(),
            ))?
            .to_string();

        let charge_sustain_name = format!("{}_Charge_Sustaining", model_name);

        let charge_sustain_model_record =
            match service.energy_model_library.get(&charge_sustain_name) {
                None => {
                    let model_names: Vec<&String> = service.energy_model_library.keys().collect();
                    return Err(TraversalModelError::BuildError(format!(
                    "No energy model found with charge_sustain_model_name = '{}', try one of: {:?}",
                    charge_sustain_name, model_names
                )));
                }
                Some(mr) => mr.clone(),
            };

        if charge_sustain_model_record
            .energy_rate_unit
            .associated_energy_unit()
            != EnergyUnit::GallonsGasoline
        {
            return Err(TraversalModelError::BuildError(format!(
                "charge_sustain_model_name = '{}' must use gasoline energy, i.e. energy_rate_unit = 'gallons_gasoline_per_mile'",
                charge_sustain_name
            )));
        }

        let charge_deplete_name = format!("{}_Charge_Depleting", model_name);

        let charge_deplete_model_record =
            match service.energy_model_library.get(&charge_deplete_name) {
                None => {
                    let model_names: Vec<&String> = service.energy_model_library.keys().collect();
                    return Err(TraversalModelError::BuildError(format!(
                    "No energy model found with charge_deplete_model_name = '{}', try one of: {:?}",
                    charge_deplete_name, model_names
                )));
                }
                Some(mr) => mr.clone(),
            };

        if charge_deplete_model_record
            .energy_rate_unit
            .associated_energy_unit()
            != EnergyUnit::KilowattHours
        {
            return Err(TraversalModelError::BuildError(format!(
                "charge_deplete_model_name = '{}' must use electrical energy, i.e. energy_rate_unit = 'kilowatt_hours_per_mile'",
                charge_deplete_name
            )));
        }

        let starting_soc_percent = conf
            .get("starting_soc_percent".to_string())
            .ok_or(TraversalModelError::BuildError(
                "No 'starting_soc_percent' key provided in query".to_string(),
            ))?
            .as_f64()
            .ok_or(TraversalModelError::BuildError(
                "Expected 'starting_soc_percent' value to be numeric".to_string(),
            ))?;

        let battery_capacity = charge_deplete_model_record.battery_capacity.ok_or(
            TraversalModelError::InternalError(
                "battery capacity not set on PHEV model".to_string(),
            ),
        )?;

        let starting_battery_energy = battery_capacity * (starting_soc_percent / 100.0);

        Ok(SpeedGradePHEVModel {
            service,
            charge_sustain_model_record,
            charge_deplete_model_record,
            energy_cost_coefficient,
            starting_battery_energy,
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
    let new_battery_energy = (current_battery_energy - electrical_energy).max(Energy::new(0.0));
    let mut updated_state = state.clone();
    updated_state[0] = state[0] + distance.into();
    updated_state[1] = state[1] + time.into();
    updated_state[2] = state[2] + electrical_energy.into();
    updated_state[3] = state[3] + gasoline_energy.into();
    updated_state[4] = new_battery_energy.into();
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
        (remaining_battery_energy_kwh.as_f64() / battery_capacity_kwh.as_f64()) * 100.0;
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
        Ok((
            pred_energy,
            pred_energy_unit,
            Energy::new(0.0),
            gasoline_energy_unit,
        ))
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
        Ok((
            Energy::new(0.0),
            electrical_energy_unit,
            pred_energy,
            pred_energy_unit,
        ))
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

    fn mock_model(conf: serde_json::Value) -> SpeedGradePHEVModel {
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
        let charge_sustain_model_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("routee")
            .join("test")
            .join("2016_CHEVROLET_Volt_Charge_Sustaining.bin");
        let charge_deplete_model_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("routee")
            .join("test")
            .join("2016_CHEVROLET_Volt_Charge_Depleting.bin");

        let charge_sustain_model_record = SpeedGradePredictionModelRecord::new(
            "Chevy_Volt_Charge_Sustaining".to_string(),
            &charge_sustain_model_file_path,
            ModelType::Smartcore,
            SpeedUnit::MilesPerHour,
            GradeUnit::Decimal,
            EnergyRateUnit::GallonsGasolinePerMile,
            Some(EnergyRate::new(0.02)),
            Some(1.1252),
            None,
            None,
        )
        .unwrap();
        let charge_deplete_model_record = SpeedGradePredictionModelRecord::new(
            "Chevy_Volt_Charge_Depleting".to_string(),
            &charge_deplete_model_file_path,
            ModelType::Smartcore,
            SpeedUnit::MilesPerHour,
            GradeUnit::Decimal,
            EnergyRateUnit::KilowattHoursPerMile,
            Some(EnergyRate::new(0.2)),
            Some(1.3958),
            Some(Energy::new(12.0)),
            Some(EnergyUnit::KilowattHours),
        )
        .unwrap();
        let mut model_library = HashMap::new();
        model_library.insert(
            "Chevy_Volt_Charge_Depleting".to_string(),
            Arc::new(charge_deplete_model_record),
        );
        model_library.insert(
            "Chevy_Volt_Charge_Sustaining".to_string(),
            Arc::new(charge_sustain_model_record),
        );

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
        SpeedGradePHEVModel::try_from((arc_service, &conf)).unwrap()
    }

    fn mock_vertex() -> Vertex {
        Vertex {
            vertex_id: VertexId(0),
            coordinate: coord! {x: -86.67, y: 36.12},
        }
    }
    fn mock_edge(distance_meters: f64) -> Edge {
        Edge {
            edge_id: EdgeId(0),
            src_vertex_id: VertexId(0),
            dst_vertex_id: VertexId(1),
            distance: Distance::new(distance_meters),
        }
    }

    #[test]
    fn test_phev_energy_model_just_electric() {
        let conf = serde_json::json!({
            "model_name": "Chevy_Volt",
            "starting_soc_percent": 100.0,
            "energy_cost_coefficient": 0.5,
        });

        let model = mock_model(conf);
        let initial = model.initial_state();

        // starting at 100% SOC, we should be able to traverse 1000 meters
        // without using any gasoline
        let e1 = mock_edge(1000.0);
        let v = mock_vertex();

        let result = model.traversal_cost(&v, &e1, &v, &initial).unwrap();
        let gasoline_energy = get_gasoline_energy_from_state(&result.updated_state);
        assert!(gasoline_energy.as_f64() < 1e-9);

        let electrical_energy = get_electrical_energy_from_state(&result.updated_state);
        assert!(electrical_energy.as_f64() > 0.0);

        let battery_percent_soc = get_battery_soc_percent(&model, &result.updated_state).unwrap();
        assert!(battery_percent_soc < 100.0);
    }

    #[test]
    fn test_phev_energy_model_gas_and_electric() {
        let conf = serde_json::json!({
            "model_name": "Chevy_Volt",
            "starting_soc_percent": 100.0,
            "energy_cost_coefficient": 0.5,
        });

        let model = mock_model(conf);
        let initial = model.initial_state();

        // now let's traverse a really long link to deplete the battery
        let distance_meters = 100.0 * 1609.344; // 100 miles
        let e1 = mock_edge(distance_meters);
        let v = mock_vertex();

        let result = model.traversal_cost(&v, &e1, &v, &initial).unwrap();
        let electrical_energy = get_electrical_energy_from_state(&result.updated_state);
        let battery_percent_soc = get_battery_soc_percent(&model, &result.updated_state).unwrap();

        assert!(electrical_energy.as_f64() > 0.0);
        assert!(battery_percent_soc < 1e-9);

        // and then traverse the same distance but this time we should only use gasoline energy
        let result2 = model
            .traversal_cost(&v, &e1, &v, &result.updated_state)
            .unwrap();

        let gasoline_energy = get_gasoline_energy_from_state(&result2.updated_state);

        assert!(gasoline_energy.as_f64() > 0.0);
    }
}
