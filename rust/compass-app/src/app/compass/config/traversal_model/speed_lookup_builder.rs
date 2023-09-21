use crate::app::compass::compass_configuration_field::CompassConfigurationField;
use crate::app::compass::config::builders::TraversalModelService;
use crate::app::compass::config::{
    builders::TraversalModelBuilder, compass_configuration_error::CompassConfigurationError,
};
use compass_core::model::traversal::default::speed_lookup_model::SpeedLookupModel;
use compass_core::model::traversal::traversal_model::TraversalModel;
use compass_core::util::unit::{SpeedUnit, TimeUnit, BASE_SPEED_UNIT, BASE_TIME_UNIT};
use log;
use std::sync::Arc;

pub struct SpeedLookupBuilder {}

pub struct SpeedLookupService {
    m: Arc<SpeedLookupModel>,
}

impl TraversalModelBuilder for SpeedLookupBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModelService>, CompassConfigurationError> {
        let filename_key = String::from("filename");
        let speed_unit_key = String::from("speed_unit");
        let time_unit_key = String::from("output_time_unit");
        let traversal_key = CompassConfigurationField::Traversal.to_string();
        // todo: optional output time unit
        let filename = parameters
            .get(&filename_key)
            .ok_or(CompassConfigurationError::ExpectedFieldForComponent(
                filename_key.clone(),
                traversal_key.clone(),
            ))?
            .as_str()
            .map(String::from)
            .ok_or(CompassConfigurationError::ExpectedFieldWithType(
                filename_key.clone(),
                String::from("String"),
            ))?;

        let speed_unit = match parameters.get(&speed_unit_key) {
            None => None,
            Some(t) => {
                let tu = serde_json::from_value::<SpeedUnit>(t.clone()).map_err(|_| {
                    CompassConfigurationError::ExpectedFieldWithType(
                        speed_unit_key.clone(),
                        String::from("SpeedUnit"),
                    )
                })?;
                Some(tu)
            }
        };

        let time_unit = match parameters.get(&time_unit_key) {
            None => None,
            Some(t) => {
                let tu = serde_json::from_value::<TimeUnit>(t.clone()).map_err(|_| {
                    CompassConfigurationError::ExpectedFieldWithType(
                        time_unit_key.clone(),
                        String::from("TimeUnit"),
                    )
                })?;
                Some(tu)
            }
        };

        match speed_unit {
            None => log::info!("no speed unit provided, using '{}'", BASE_SPEED_UNIT),
            Some(_) => (),
        }
        match time_unit {
            None => log::info!("no time unit provided, using '{}'", BASE_TIME_UNIT),
            Some(_) => (),
        }

        let m = SpeedLookupModel::new(&filename, speed_unit, time_unit)
            .map_err(CompassConfigurationError::TraversalModelError)?;
        let service = Arc::new(SpeedLookupService { m: Arc::new(m) });
        return Ok(service);
    }
}

impl TraversalModelService for SpeedLookupService {
    fn build(
        &self,
        _parameters: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModel>, CompassConfigurationError> {
        return Ok(self.m.clone());
    }
}
