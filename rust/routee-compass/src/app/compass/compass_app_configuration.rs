use config::Config;
use routee_compass_core::algorithm::search::search_orientation::SearchOrientation;

use crate::app::compass::config::compass_configuration_field::CompassConfigurationField;

use super::{
    compass_app_error::CompassAppError,
    response::{
        response_output_policy::ResponseOutputPolicy,
        response_persistence_policy::ResponsePersistencePolicy,
    },
};

pub struct CompassAppConfiguration {
    pub parallelism: usize,
    pub search_orientation: SearchOrientation,
    pub response_persistence_policy: ResponsePersistencePolicy,
    pub response_output_policy: ResponseOutputPolicy,
}

impl CompassAppConfiguration {
    pub fn new(
        parallelism: usize,
        search_orientation: SearchOrientation,
        response_persistence_policy: ResponsePersistencePolicy,
        response_output_policy: ResponseOutputPolicy,
    ) -> CompassAppConfiguration {
        CompassAppConfiguration {
            parallelism,
            search_orientation,
            response_persistence_policy,
            response_output_policy,
        }
    }
}

impl TryFrom<&Config> for CompassAppConfiguration {
    type Error = CompassAppError;

    fn try_from(config: &Config) -> Result<Self, Self::Error> {
        // other parameters
        let parallelism = config.get::<usize>(CompassConfigurationField::Parallelism.to_str())?;
        let search_orientation = config
            .get::<SearchOrientation>(CompassConfigurationField::SearchOrientation.to_str())?;
        let response_persistence_policy = config.get::<ResponsePersistencePolicy>(
            CompassConfigurationField::ResponsePersistencePolicy.to_str(),
        )?;
        let response_output_policy = config.get::<ResponseOutputPolicy>(
            CompassConfigurationField::ResponseOutputPolicy.to_str(),
        )?;
        let configuration = CompassAppConfiguration::new(
            parallelism,
            search_orientation,
            response_persistence_policy,
            response_output_policy,
        );

        Ok(configuration)
    }
}
