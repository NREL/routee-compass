use std::sync::Arc;

use crate::app::compass::compass_configuration_field::CompassConfigurationField;
use crate::app::compass::config::builders::TraversalModelService;
use crate::app::compass::config::{
    builders::TraversalModelBuilder, compass_configuration_error::CompassConfigurationError,
};
use compass_core::model::traversal::traversal_model::TraversalModel;
use compass_core::util::unit::{EnergyRateUnit, SpeedUnit};
use compass_core::util::unit::{EnergyUnit, TimeUnit};
use compass_powertrain::routee::smart_core_energy_model::SmartCoreEnergyModel;

pub struct EnergyModelBuilder {}

pub struct EnergyModelService {
    m: Arc<SmartCoreEnergyModel>,
}

impl TraversalModelBuilder for EnergyModelBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModelService>, CompassConfigurationError> {
        let velocity_filename_key = String::from("speeds_filename");
        let routee_filename_key = String::from("routee_filename");

        let speed_table_speed_unit_key = String::from("speed_table_speed_unit");
        let routee_model_speed_unit_key = String::from("routee_model_speed_unit");
        let routee_model_energy_rate_unit_key = String::from("routee_model_energy_rate_unit");
        let energy_percent_key = String::from("energy_percent");
        let traversal_key = CompassConfigurationField::Traversal.to_string();

        let velocity_filename = parameters
            .get(&velocity_filename_key)
            .ok_or(CompassConfigurationError::ExpectedFieldForComponent(
                velocity_filename_key.clone(),
                traversal_key.clone(),
            ))?
            .as_str()
            .map(String::from)
            .ok_or(CompassConfigurationError::ExpectedFieldWithType(
                velocity_filename_key.clone(),
                String::from("String"),
            ))?;

        let routee_filename = parameters
            .get(&routee_filename_key)
            .ok_or(CompassConfigurationError::ExpectedFieldForComponent(
                routee_filename_key.clone(),
                traversal_key.clone(),
            ))?
            .as_str()
            .map(String::from)
            .ok_or(CompassConfigurationError::ExpectedFieldWithType(
                routee_filename_key.clone(),
                String::from("String"),
            ))?;

        let speed_table_speed_unit = parameters
            .get(&speed_table_speed_unit_key)
            .map(|t| serde_json::from_value::<SpeedUnit>(t.clone()))
            .ok_or(CompassConfigurationError::ExpectedFieldForComponent(
                velocity_filename_key.clone(),
                speed_table_speed_unit_key.clone(),
            ))?
            .map_err(CompassConfigurationError::SerdeDeserializationError)?;

        let routee_model_speed_unit = parameters
            .get(&routee_model_speed_unit_key)
            .map(|t| serde_json::from_value::<SpeedUnit>(t.clone()))
            .ok_or(CompassConfigurationError::ExpectedFieldForComponent(
                velocity_filename_key.clone(),
                routee_model_speed_unit_key.clone(),
            ))?
            .map_err(CompassConfigurationError::SerdeDeserializationError)?;

        let routee_model_energy_rate_unit = parameters
            .get(&routee_model_energy_rate_unit_key)
            .map(|t| serde_json::from_value::<EnergyRateUnit>(t.clone()))
            .ok_or(CompassConfigurationError::ExpectedFieldForComponent(
                velocity_filename_key.clone(),
                routee_model_energy_rate_unit_key.clone(),
            ))?
            .map_err(CompassConfigurationError::SerdeDeserializationError)?;

        let energy_percent =
            parameters
                .get(&energy_percent_key)
                .map_or(Ok(1.0), |v| match v.as_f64() {
                    None => Err(CompassConfigurationError::ExpectedFieldWithType(
                        energy_percent_key.clone(),
                        String::from("numeric"),
                    )),
                    Some(f) if f < 0.0 || 1.0 < f => {
                        Err(CompassConfigurationError::ExpectedFieldWithType(
                            energy_percent_key.clone(),
                            String::from("decimal in [0.0, 1.0]"),
                        ))
                    }
                    Some(f) => Ok(f),
                })?;

        let m = SmartCoreEnergyModel::new(
            &velocity_filename,
            &routee_filename,
            routee_model_energy_rate_unit,
            speed_table_speed_unit,
            routee_model_speed_unit,
            energy_percent,
        )
        .map_err(CompassConfigurationError::TraversalModelError)?;
        let service = EnergyModelService { m: Arc::new(m) };
        return Ok(Arc::new(service));
    }
}

impl TraversalModelService for EnergyModelService {
    fn build(
        &self,
        _parameters: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModel>, CompassConfigurationError> {
        return Ok(self.m.clone());
    }
}
