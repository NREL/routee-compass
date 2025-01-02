use super::turn_delay_access_model::TurnDelayAccessModel;
use super::turn_delay_access_model_engine::TurnDelayAccessModelEngine;
use crate::model::access::AccessModel;
use crate::model::access::AccessModelError;
use crate::model::access::AccessModelService;
use std::sync::Arc;

pub struct TurnDelayAccessModelService {
    pub engine: Arc<TurnDelayAccessModelEngine>,
}

impl TurnDelayAccessModelService {}

impl AccessModelService for TurnDelayAccessModelService {
    fn build(&self, _query: &serde_json::Value) -> Result<Arc<dyn AccessModel>, AccessModelError> {
        let model = TurnDelayAccessModel {
            engine: self.engine.clone(),
        };
        Ok(Arc::new(model))
    }
}
