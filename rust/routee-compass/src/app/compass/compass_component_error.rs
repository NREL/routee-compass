use crate::plugin::PluginError;
use routee_compass_core::config::CompassConfigurationError;

#[derive(thiserror::Error, Debug)]
pub enum CompassComponentError {
    #[error(transparent)]
    CompassConfigurationError(#[from] CompassConfigurationError),
    #[error(transparent)]
    PluginError(#[from] PluginError),
}
