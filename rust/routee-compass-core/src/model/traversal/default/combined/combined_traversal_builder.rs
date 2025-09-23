use super::CombinedTraversalService;
use crate::model::traversal::{default::combined::CombinedTraversalConfig, TraversalModelBuilder, TraversalModelError, TraversalModelService};
use itertools::Itertools;
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
        let conf: CombinedTraversalConfig = serde_json::from_value(parameters.clone()).map_err(|e| TraversalModelError::BuildError(format!("failure reading Combined Traversal Config: {e}")))?;
        let services: Vec<Arc<dyn TraversalModelService>> = conf.models
            .iter()
            .map(|conf| build_model_from_json(conf, &self.builders))
            .try_collect()?;
        let service: Arc<dyn TraversalModelService> = Arc::new(CombinedTraversalService::new(services, conf.ignore_missing));
        Ok(service)
    }
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
            "unknown traversal model name '{key_json}', must be one of: [{valid}]"
        ))
    })?;
    b.build(conf)
}
