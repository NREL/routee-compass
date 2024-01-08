use crate::app::compass::config::{
    compass_configuration_error::CompassConfigurationError,
    compass_configuration_field::CompassConfigurationField,
    config_json_extension::ConfigJsonExtensions,
};
use itertools::Itertools;
use routee_compass_core::model::frontier::{
    frontier_model_builder::FrontierModelBuilder, frontier_model_error::FrontierModelError,
    frontier_model_service::FrontierModelService,
};
use std::{collections::HashMap, rc::Rc, sync::Arc};

use super::combined_service::CombinedFrontierService;

pub struct CombinedBuilder {
    pub builders: HashMap<String, Rc<dyn FrontierModelBuilder>>,
}

impl CombinedBuilder {
    pub fn register_builder(
        &self,
        builder_key: String,
        builder: Rc<dyn FrontierModelBuilder>,
    ) -> Self {
        let mut builders = self.builders.clone();
        builders.insert(builder_key, builder);
        CombinedBuilder { builders }
    }
    fn build_service(
        &self,
        config: &serde_json::Value,
    ) -> Result<Arc<dyn FrontierModelService>, CompassConfigurationError> {
        let fm_type_obj = config.get("type").ok_or_else(|| {
            CompassConfigurationError::ExpectedFieldForComponent(
                CompassConfigurationField::Frontier.to_string(),
                String::from("type"),
            )
        })?;
        let fm_type: String = fm_type_obj
            .as_str()
            .ok_or_else(|| {
                CompassConfigurationError::ExpectedFieldWithType(
                    String::from("type"),
                    String::from("String"),
                )
            })?
            .into();
        self.builders
            .get(&fm_type)
            .ok_or_else(|| {
                CompassConfigurationError::UnknownModelNameForComponent(
                    fm_type.clone(),
                    String::from("frontier"),
                    self.builders.keys().join(", "),
                )
            })
            .and_then(|b| {
                b.build(config)
                    .map_err(CompassConfigurationError::FrontierModelError)
            })
    }
}

impl FrontierModelBuilder for CombinedBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn FrontierModelService>, FrontierModelError> {
        let frontier_key = CompassConfigurationField::Frontier;
        let params = parameters
            .get_config_array(&"models", &frontier_key)
            .map_err(|e| FrontierModelError::BuildError(e.to_string()))?;

        let inner_services = params
            .iter()
            .map(|p| self.build_service(p))
            .collect::<Result<Vec<Arc<dyn FrontierModelService>>, CompassConfigurationError>>()
            .map_err(|e| FrontierModelError::BuildError(e.to_string()))?;

        let service = CombinedFrontierService { inner_services };

        Ok(Arc::new(service))
    }
}
