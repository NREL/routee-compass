use super::{GradeTraversalEngine, GradeTraversalModel};
use crate::model::{
    state::{InputFeature, StateModel, StateVariableConfig},
    traversal::{default::fieldname, TraversalModel, TraversalModelError, TraversalModelService},
    unit::RatioUnit,
};
use std::sync::Arc;
use uom::{si::f64::Ratio, ConstZero};

pub struct GradeTraversalService {
    engine: Arc<GradeTraversalEngine>,
}

impl GradeTraversalService {
    pub fn new(engine: Arc<GradeTraversalEngine>) -> GradeTraversalService {
        GradeTraversalService { engine }
    }
}

impl TraversalModelService for GradeTraversalService {
    fn input_features(&self) -> Vec<InputFeature> {
        vec![]
    }

    fn output_features(&self) -> Vec<(String, StateVariableConfig)> {
        vec![(
            String::from(fieldname::EDGE_GRADE),
            StateVariableConfig::Ratio {
                initial: Ratio::ZERO,
                accumulator: false,
                output_unit: Some(RatioUnit::default()),
            },
        )]
    }

    fn build(
        &self,
        _query: &serde_json::Value,
        state_model: Arc<StateModel>,
    ) -> Result<Arc<dyn TraversalModel>, TraversalModelError> {
        let edge_grade_idx = state_model.get_index(fieldname::EDGE_GRADE).map_err(|e| {
            TraversalModelError::BuildError(format!("Failed to find EDGE_GRADE index: {}", e))
        })?;

        let model = GradeTraversalModel::new(self.engine.clone(), edge_grade_idx);
        Ok(Arc::new(model))
    }
}
