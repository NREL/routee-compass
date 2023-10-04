use crate::app::compass::config::config_json_extension::ConfigJsonExtensions;
use crate::app::compass::{
    compass_configuration_field::CompassConfigurationField,
    config::{
        builders::{TraversalModelBuilder, TraversalModelService},
        compass_configuration_error::CompassConfigurationError,
    },
};
use compass_core::model::traversal::traversal_model::TraversalModel;
use compass_core::{model::traversal::default::distance::DistanceModel, util::unit::DistanceUnit};
use std::sync::Arc;

pub struct DistanceBuilder {}

pub struct DistanceService {}

impl TraversalModelBuilder for DistanceBuilder {
    fn build(
        &self,
        _parameters: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModelService>, CompassConfigurationError> {
        let m: Arc<dyn TraversalModelService> = Arc::new(DistanceService {});
        return Ok(m);
    }
}

impl TraversalModelService for DistanceService {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModel>, CompassConfigurationError> {
        let traversal_key = CompassConfigurationField::Traversal.to_string();
        let distance_unit_option = parameters.get_config_serde_optional::<DistanceUnit>(
            String::from("distance_unit"),
            traversal_key.clone(),
        )?;
        let m: Arc<dyn TraversalModel> = Arc::new(DistanceModel::new(distance_unit_option));
        return Ok(m);
    }
}
