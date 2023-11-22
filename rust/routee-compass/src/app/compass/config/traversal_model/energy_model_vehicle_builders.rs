use std::sync::Arc;

use routee_compass_core::{
    model::traversal::traversal_model_error::TraversalModelError,
    util::unit::{Energy, EnergyRate, EnergyRateUnit, EnergyUnit, GradeUnit, SpeedUnit},
};
use routee_compass_powertrain::routee::{
    prediction::{load_prediction_model, model_type::ModelType, PredictionModelRecord},
    vehicles::{
        default::{conventional::ConventionalVehicle, plug_in_hybrid::PlugInHybrid},
        Vehicle,
    },
};

use crate::app::compass::{
    compass_app_error::CompassAppError,
    config::{
        compass_configuration_error::CompassConfigurationError,
        compass_configuration_field::CompassConfigurationField,
        config_json_extension::ConfigJsonExtensions,
    },
};

pub trait VehicleBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn Vehicle>, CompassConfigurationError>;
}

pub struct ConventionalVehicleBuilder {}
pub struct PlugInHybridBuilder {}

impl VehicleBuilder for ConventionalVehicleBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn Vehicle>, CompassConfigurationError> {
        let vehicle_key = String::from("vehicles");
        let name = parameters.get_config_string(String::from("name"), vehicle_key.clone())?;

        let model_record = get_model_record_from_params(parameters)?;

        let vehicle = ConventionalVehicle::new(name, model_record)?;

        Ok(Arc::new(vehicle))
    }
}

impl VehicleBuilder for PlugInHybridBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn Vehicle>, CompassConfigurationError> {
        let vehicle_key = String::from("vehicles");
        let name = parameters.get_config_string(String::from("name"), vehicle_key.clone())?;

        let charge_deplete_params =
            parameters.get_config_section(CompassConfigurationField::ChargeDeplete)?;
        let charge_deplete_record = get_model_record_from_params(&charge_deplete_params)?;
        let charge_sustain_params =
            parameters.get_config_section(CompassConfigurationField::ChargeSustain)?;
        let charge_sustain_record = get_model_record_from_params(&charge_sustain_params)?;

        let battery_capacity = parameters
            .get_config_serde::<Energy>(String::from("battery_capacity"), vehicle_key.clone())?;
        let battery_energy_unit = parameters.get_config_serde::<EnergyUnit>(
            String::from("battery_energy_unit"),
            vehicle_key.clone(),
        )?;
        let starting_battery_energy = battery_capacity;
        let phev = PlugInHybrid::new(
            name,
            charge_deplete_record,
            charge_sustain_record,
            battery_capacity,
            starting_battery_energy,
            battery_energy_unit,
        )?;
        Ok(Arc::new(phev))
    }
}

fn get_model_record_from_params(
    parameters: &serde_json::Value,
) -> Result<PredictionModelRecord, CompassConfigurationError> {
    let vehicle_key = String::from("vehicles");
    let name = parameters.get_config_string(String::from("name"), vehicle_key.clone())?;
    let model_path =
        parameters.get_config_path(String::from("model_input_file"), vehicle_key.clone())?;
    let model_type = parameters
        .get_config_serde::<ModelType>(String::from("model_type"), vehicle_key.clone())?;
    let speed_unit = parameters
        .get_config_serde::<SpeedUnit>(String::from("speed_unit"), vehicle_key.clone())?;
    let ideal_energy_rate_option = parameters.get_config_serde_optional::<EnergyRate>(
        String::from("ideal_energy_rate"),
        vehicle_key.clone(),
    )?;
    let grade_unit = parameters
        .get_config_serde::<GradeUnit>(String::from("grade_unit"), vehicle_key.clone())?;

    let energy_rate_unit = parameters.get_config_serde::<EnergyRateUnit>(
        String::from("energy_rate_unit"),
        vehicle_key.clone(),
    )?;
    let real_world_energy_adjustment_option = parameters.get_config_serde_optional::<f64>(
        String::from("real_world_energy_adjustment"),
        vehicle_key.clone(),
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
