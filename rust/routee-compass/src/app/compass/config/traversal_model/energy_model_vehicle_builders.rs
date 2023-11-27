use std::sync::Arc;

use routee_compass_core::util::unit::{
    Energy, EnergyRate, EnergyRateUnit, EnergyUnit, GradeUnit, SpeedUnit,
};
use routee_compass_powertrain::routee::{
    prediction::{load_prediction_model, model_type::ModelType, PredictionModelRecord},
    vehicle::{
        default::{dual_fuel_vehicle::DualFuelVehicle, single_fuel_vehicle::SingleFuelVehicle},
        VehicleType,
    },
};

use crate::app::compass::config::{
    compass_configuration_error::CompassConfigurationError,
    compass_configuration_field::CompassConfigurationField,
    config_json_extension::ConfigJsonExtensions,
};

pub enum VehicleBuilder {
    SingleFuel,
    DualFuel,
}

impl VehicleBuilder {
    pub fn from_string(vehicle_type: String) -> Result<VehicleBuilder, CompassConfigurationError> {
        match vehicle_type.as_str() {
            "single_fuel" => Ok(VehicleBuilder::SingleFuel),
            "dual_fuel" => Ok(VehicleBuilder::DualFuel),
            _ => Err(CompassConfigurationError::ExpectedFieldWithType(
                "vehicle.type".to_string(),
                "string".to_string(),
            )),
        }
    }
    pub fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn VehicleType>, CompassConfigurationError> {
        match self {
            VehicleBuilder::SingleFuel => build_conventional(parameters),
            VehicleBuilder::DualFuel => build_plugin_hybrid(parameters),
        }
    }
}

fn build_conventional(
    parameters: &serde_json::Value,
) -> Result<Arc<dyn VehicleType>, CompassConfigurationError> {
    let vehicle_key = String::from("single_fuel");
    let name = parameters.get_config_string(String::from("name"), vehicle_key.clone())?;

    let model_record = get_model_record_from_params(parameters, name.clone())?;

    let vehicle = SingleFuelVehicle::new(name, model_record)?;

    Ok(Arc::new(vehicle))
}

fn build_plugin_hybrid(
    parameters: &serde_json::Value,
) -> Result<Arc<dyn VehicleType>, CompassConfigurationError> {
    let vehicle_key = String::from("dual_fuel");
    let name = parameters.get_config_string(String::from("name"), vehicle_key.clone())?;

    let charge_depleting_params =
        parameters.get_config_section(CompassConfigurationField::ChargeDepleting)?;

    let charge_depleting_record = get_model_record_from_params(
        &charge_depleting_params,
        format!("charge_depleting: {}", name.clone()),
    )?;
    let charge_sustain_params =
        parameters.get_config_section(CompassConfigurationField::ChargeSustaining)?;

    let charge_sustain_record = get_model_record_from_params(
        &charge_sustain_params,
        format!("charge_sustain: {}", name.clone()),
    )?;

    let battery_capacity = parameters
        .get_config_serde::<Energy>(String::from("battery_capacity"), vehicle_key.clone())?;
    let battery_energy_unit = parameters.get_config_serde::<EnergyUnit>(
        String::from("battery_capacity_unit"),
        vehicle_key.clone(),
    )?;
    let starting_battery_energy = battery_capacity;
    let phev = DualFuelVehicle::new(
        name,
        charge_sustain_record,
        charge_depleting_record,
        battery_capacity,
        starting_battery_energy,
        battery_energy_unit,
    )?;
    Ok(Arc::new(phev))
}

fn get_model_record_from_params(
    parameters: &serde_json::Value,
    parent_key: String,
) -> Result<PredictionModelRecord, CompassConfigurationError> {
    let name = parameters.get_config_string(String::from("name"), parent_key.clone())?;
    let model_path =
        parameters.get_config_path(String::from("model_input_file"), parent_key.clone())?;
    let model_type =
        parameters.get_config_serde::<ModelType>(String::from("model_type"), parent_key.clone())?;
    let speed_unit =
        parameters.get_config_serde::<SpeedUnit>(String::from("speed_unit"), parent_key.clone())?;
    let ideal_energy_rate_option = parameters.get_config_serde_optional::<EnergyRate>(
        String::from("ideal_energy_rate"),
        parent_key.clone(),
    )?;
    let grade_unit =
        parameters.get_config_serde::<GradeUnit>(String::from("grade_unit"), parent_key.clone())?;

    let energy_rate_unit = parameters
        .get_config_serde::<EnergyRateUnit>(String::from("energy_rate_unit"), parent_key.clone())?;
    let real_world_energy_adjustment_option = parameters.get_config_serde_optional::<f64>(
        String::from("real_world_energy_adjustment"),
        parent_key.clone(),
    )?;

    let model_record = load_prediction_model(
        name.clone(),
        &model_path,
        model_type,
        speed_unit,
        grade_unit,
        energy_rate_unit,
        ideal_energy_rate_option,
        real_world_energy_adjustment_option,
    )?;

    Ok(model_record)
}
