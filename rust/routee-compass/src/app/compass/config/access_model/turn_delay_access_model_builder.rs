use std::sync::Arc;

use routee_compass_core::model::access::{
    access_model_builder::AccessModelBuilder, access_model_error::AccessModelError,
    access_model_service::AccessModelService,
};

pub struct TurnDelayAccessModelBuilder {}

impl AccessModelBuilder for TurnDelayAccessModelBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn AccessModelService>, AccessModelError> {
        todo!()
    }
}
