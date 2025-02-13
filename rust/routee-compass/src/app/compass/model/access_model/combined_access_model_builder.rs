use itertools::Itertools;
use routee_compass_core::config::ConfigJsonExtensions;
use routee_compass_core::model::access::{
    default::CombinedAccessModelService, AccessModelBuilder, AccessModelError, AccessModelService,
};
use std::{collections::HashMap, rc::Rc, sync::Arc};

pub struct CombinedAccessModelBuilder {
    pub builders: HashMap<String, Rc<dyn AccessModelBuilder>>,
}

impl AccessModelBuilder for CombinedAccessModelBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn AccessModelService>, AccessModelError> {
        let model_params = parameters
            .get_config_array(&"access_models", &"combined")
            .map_err(|e| {
                AccessModelError::BuildError(format!(
                    "unable to decode combined.access_models: {}",
                    e
                ))
            })?;
        let services = model_params
            .iter()
            .map(|params| {
                let model_type = params
                    .get_config_string(&"type", &"combined.access_models")
                    .map_err(|e| {
                        AccessModelError::BuildError(format!(
                            "unable to find 'type' of combined.access_model listing: {}",
                            e
                        ))
                    })?;
                let builder = self.builders.get(&model_type).ok_or_else(|| {
                    let alts = self.builders.keys().join(",");
                    AccessModelError::BuildError(format!(
                        "unregistered access model {}, should be one of: {{{}}}",
                        model_type, alts
                    ))
                })?;
                builder.build(params)
            })
            .collect::<Result<_, _>>()?;
        Ok(Arc::new(CombinedAccessModelService { services }))
    }
}
