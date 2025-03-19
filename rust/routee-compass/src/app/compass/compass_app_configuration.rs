use super::{
    response::{
        response_output_policy::ResponseOutputPolicy,
        response_persistence_policy::ResponsePersistencePolicy,
    },
    CompassAppError,
};
use config::Config;
use routee_compass_core::config::CompassConfigurationField;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CompassAppConfiguration {
    pub parallelism: usize,
    pub response_persistence_policy: ResponsePersistencePolicy,
    pub response_output_policy: ResponseOutputPolicy,
}

impl CompassAppConfiguration {
    pub fn new(
        parallelism: usize,
        response_persistence_policy: ResponsePersistencePolicy,
        response_output_policy: ResponseOutputPolicy,
    ) -> CompassAppConfiguration {
        CompassAppConfiguration {
            parallelism,
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
        let response_persistence_policy = config.get::<ResponsePersistencePolicy>(
            CompassConfigurationField::ResponsePersistencePolicy.to_str(),
        )?;
        let response_output_policy = config.get::<ResponseOutputPolicy>(
            CompassConfigurationField::ResponseOutputPolicy.to_str(),
        )?;
        let configuration = CompassAppConfiguration::new(
            parallelism,
            response_persistence_policy,
            response_output_policy,
        );

        Ok(configuration)
    }
}
