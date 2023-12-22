use crate::app::compass::config::{
    compass_configuration_error::CompassConfigurationError,
    compass_configuration_field::CompassConfigurationField,
    config_json_extension::ConfigJsonExtensions,
};
use routee_compass_core::model::unit::{
    Energy, EnergyRate, EnergyRateUnit, EnergyUnit, GradeUnit, SpeedUnit,
};
use routee_compass_core::util::cache_policy::float_cache_policy::{
    FloatCachePolicy, FloatCachePolicyConfig,
};
use routee_compass_powertrain::routee::{
    prediction::{load_prediction_model, model_type::ModelType, PredictionModelRecord},
    vehicle::{
        default::{bev::BEV, ice::ICE, phev::PHEV},
        VehicleType,
    },
};
use std::sync::Arc;

pub enum VehicleBuilder {
    ICE,
    BEV,
    PHEV,
}

impl VehicleBuilder {
    pub fn from_string(vehicle_type: String) -> Result<VehicleBuilder, CompassConfigurationError> {
        match vehicle_type.as_str() {
            "ice" => Ok(VehicleBuilder::ICE),
            "bev" => Ok(VehicleBuilder::BEV),
            "phev" => Ok(VehicleBuilder::PHEV),
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
            VehicleBuilder::ICE => build_conventional(parameters),
            VehicleBuilder::BEV => build_battery_electric(parameters),
            VehicleBuilder::PHEV => build_plugin_hybrid(parameters),
        }
    }
}

fn build_conventional(
    parameters: &serde_json::Value,
) -> Result<Arc<dyn VehicleType>, CompassConfigurationError> {
    let vehicle_key = String::from("ice");
    let name = parameters.get_config_string(&"name", &vehicle_key)?;

    let model_record = get_model_record_from_params(parameters, &name)?;

    let vehicle = ICE::new(name, model_record)?;

    Ok(Arc::new(vehicle))
}

fn build_battery_electric(
    parameters: &serde_json::Value,
) -> Result<Arc<dyn VehicleType>, CompassConfigurationError> {
    let name = parameters.get_config_string(&"name", &"bev")?;

    let model_record = get_model_record_from_params(parameters, &name)?;

    let battery_capacity = parameters.get_config_serde::<Energy>(&"battery_capacity", &"bev")?;
    let battery_energy_unit =
        parameters.get_config_serde::<EnergyUnit>(&"battery_capacity_unit", &"bev")?;
    let starting_battery_energy = battery_capacity;

    let vehicle = BEV::new(
        name,
        model_record,
        battery_capacity,
        starting_battery_energy,
        battery_energy_unit,
    );

    Ok(Arc::new(vehicle))
}

fn build_plugin_hybrid(
    parameters: &serde_json::Value,
) -> Result<Arc<dyn VehicleType>, CompassConfigurationError> {
    let name = parameters.get_config_string(&"name", &"phev")?;

    let charge_depleting_params =
        parameters.get_config_section(CompassConfigurationField::ChargeDepleting)?;

    let charge_depleting_record = get_model_record_from_params(
        &charge_depleting_params,
        &format!("charge_depleting: {}", &name),
    )?;
    let charge_sustain_params =
        parameters.get_config_section(CompassConfigurationField::ChargeSustaining)?;

    let charge_sustain_record = get_model_record_from_params(
        &charge_sustain_params,
        &format!("charge_sustain: {}", &name),
    )?;

    let battery_capacity = parameters.get_config_serde::<Energy>(&"battery_capacity", &"phev")?;
    let battery_energy_unit =
        parameters.get_config_serde::<EnergyUnit>(&"battery_capacity_unit", &"phev")?;

    let custom_liquid_fuel_to_kwh =
        parameters.get_config_serde_optional::<f64>(&"custom_liquid_fuel_to_kwh", &"phev")?;
    let starting_battery_energy = battery_capacity;
    let phev = PHEV::new(
        name,
        charge_sustain_record,
        charge_depleting_record,
        battery_capacity,
        starting_battery_energy,
        battery_energy_unit,
        custom_liquid_fuel_to_kwh,
    )?;
    Ok(Arc::new(phev))
}

fn get_model_record_from_params(
    parameters: &serde_json::Value,
    parent_key: &String,
) -> Result<PredictionModelRecord, CompassConfigurationError> {
    let name = parameters.get_config_string(&"name", &parent_key)?;
    let model_path = parameters.get_config_path(&"model_input_file", &parent_key)?;
    let model_type = parameters.get_config_serde::<ModelType>(&"model_type", &parent_key)?;
    let speed_unit = parameters.get_config_serde::<SpeedUnit>(&"speed_unit", &parent_key)?;
    let ideal_energy_rate_option =
        parameters.get_config_serde_optional::<EnergyRate>(&"ideal_energy_rate", &parent_key)?;
    let grade_unit = parameters.get_config_serde::<GradeUnit>(&"grade_unit", &parent_key)?;

    let energy_rate_unit =
        parameters.get_config_serde::<EnergyRateUnit>(&"energy_rate_unit", &parent_key)?;
    let real_world_energy_adjustment_option = parameters
        .get_config_serde_optional::<f64>(&"real_world_energy_adjustment", &parent_key)?;

    let cache_config = parameters
        .get_config_serde_optional::<FloatCachePolicyConfig>(&"float_cache_policy", parent_key)?;

    let cache = match cache_config {
        Some(config) => Some(FloatCachePolicy::from_config(config)?),
        None => None,
    };

    let model_record = load_prediction_model(
        name.clone(),
        &model_path,
        model_type,
        speed_unit,
        grade_unit,
        energy_rate_unit,
        ideal_energy_rate_option,
        real_world_energy_adjustment_option,
        cache,
    )?;

    Ok(model_record)
}
