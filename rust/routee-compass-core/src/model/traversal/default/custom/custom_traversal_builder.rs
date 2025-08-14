use super::{CustomTraversalConfig, CustomTraversalEngine, CustomTraversalService};
use crate::model::traversal::TraversalModelBuilder;
use crate::model::traversal::TraversalModelError;
use crate::model::traversal::TraversalModelService;
use std::sync::Arc;

pub struct CustomTraversalBuilder {}

impl TraversalModelBuilder for CustomTraversalBuilder {
    fn build(
        &self,
        params: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModelService>, TraversalModelError> {
        let config: CustomTraversalConfig =
            serde_json::from_value(params.clone()).map_err(|e| {
                TraversalModelError::BuildError(format!(
                    "failure building custom traversal model service: {e}"
                ))
            })?;

        let engine = CustomTraversalEngine::try_from(&config)?;
        let service = Arc::new(CustomTraversalService {
            engine: Arc::new(engine),
        });
        Ok(service)
    }
}
