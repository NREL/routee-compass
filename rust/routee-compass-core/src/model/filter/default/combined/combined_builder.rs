use crate::config::{CompassConfigurationError, CompassConfigurationField, ConfigJsonExtensions};
use crate::model::filter::{FilterModelBuilder, FilterModelError, FilterModelService};
use itertools::Itertools;
use std::{collections::HashMap, rc::Rc, sync::Arc};

use super::combined_service::CombinedFrontierService;

pub struct CombinedFilterModelBuilder {
    pub builders: HashMap<String, Rc<dyn FilterModelBuilder>>,
}

impl CombinedFilterModelBuilder {
    pub fn new(builders: HashMap<String, Rc<dyn FilterModelBuilder>>) -> Self {
        Self { builders }
    }

    fn build_service(
        &self,
        config: &serde_json::Value,
    ) -> Result<Arc<dyn FilterModelService>, CompassConfigurationError> {
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
                    String::from("filter"),
                    self.builders.keys().join(", "),
                )
            })
            .and_then(|b| {
                b.build(config)
                    .map_err(CompassConfigurationError::FilterModelError)
            })
    }
}

impl FilterModelBuilder for CombinedFilterModelBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn FilterModelService>, FilterModelError> {
        let filter_key = CompassConfigurationField::Frontier;
        let params = parameters
            .get_config_array(&"models", &filter_key)
            .map_err(|e| FilterModelError::BuildError(e.to_string()))?;

        let inner_services = params
            .iter()
            .map(|p| self.build_service(p))
            .collect::<Result<Vec<Arc<dyn FilterModelService>>, CompassConfigurationError>>()
            .map_err(|e| FilterModelError::BuildError(e.to_string()))?;

        let service = CombinedFrontierService { inner_services };

        Ok(Arc::new(service))
    }
}
