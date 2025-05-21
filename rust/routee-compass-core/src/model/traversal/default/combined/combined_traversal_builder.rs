use super::CombinedTraversalService;
use crate::model::traversal::{TraversalModelBuilder, TraversalModelError, TraversalModelService};
use itertools::Itertools;
use log;
use std::{collections::HashMap, rc::Rc, sync::Arc};

pub struct CombinedTraversalBuilder {
    builders: HashMap<String, Rc<dyn TraversalModelBuilder>>,
}

impl CombinedTraversalBuilder {
    pub fn new(
        builders: HashMap<String, Rc<dyn TraversalModelBuilder>>,
    ) -> CombinedTraversalBuilder {
        CombinedTraversalBuilder { builders }
    }
}

impl TraversalModelBuilder for CombinedTraversalBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModelService>, TraversalModelError> {
        match parameters.get("models") {
            None => {
                let model_names = self.builders.keys().join(", ");
                log::info!("no model selection provided, attempting to build all models in collection: [{}]", model_names);
                build_all_models(parameters, &self.builders)
            }
            Some(conf) => build_selected_models(conf, &self.builders),
        }
    }
}

fn build_selected_models(
    conf: &serde_json::Value,
    builders: &HashMap<String, Rc<dyn TraversalModelBuilder>>,
) -> Result<Arc<dyn TraversalModelService>, TraversalModelError> {
    let models_vec = conf.as_array().ok_or_else(|| {
        TraversalModelError::BuildError(format!(
            "combined traversal model found key 'models' but was not an array, found '{}'",
            serde_json::to_string(conf).unwrap_or_default()
        ))
    })?;
    let services: Vec<Arc<dyn TraversalModelService>> = models_vec
        .iter()
        .map(|conf| build_model_from_json(conf, builders))
        .try_collect()?;
    let service: Arc<dyn TraversalModelService> = Arc::new(CombinedTraversalService::new(services));
    Ok(service)
}

fn build_all_models(
    conf: &serde_json::Value,
    builders: &HashMap<String, Rc<dyn TraversalModelBuilder>>,
) -> Result<Arc<dyn TraversalModelService>, TraversalModelError> {
    let services: Vec<Arc<dyn TraversalModelService>> = builders
        .values()
        .map(|builder| builder.build(conf))
        .try_collect()?;
    let service: Arc<dyn TraversalModelService> = Arc::new(CombinedTraversalService::new(services));
    Ok(service)
}

/// builds a model from its configuration within the combined traversal model
fn build_model_from_json(
    conf: &serde_json::Value,
    builders: &HashMap<String, Rc<dyn TraversalModelBuilder>>,
) -> Result<Arc<dyn TraversalModelService>, TraversalModelError> {
    let key_json = conf.get("type").ok_or_else(|| {
        TraversalModelError::BuildError(format!(
            "traversal model configuration missing 'type' keyword: '{}'",
            serde_json::to_string(conf).unwrap_or_default()
        ))
    })?;
    let key = key_json.as_str().ok_or_else(|| {
        TraversalModelError::BuildError(format!(
            "expected key 'type' to point to a string, found '{}'",
            serde_json::to_string(key_json).unwrap_or_default()
        ))
    })?;
    let b = builders.get(key).ok_or_else(|| {
        let valid = builders.keys().join(", ");
        TraversalModelError::BuildError(format!(
            "unknown traversal model name '{}', must be one of: [{}]",
            key_json, valid
        ))
    })?;
    b.build(conf)
}
