use super::{GradeTraversalEngine, GradeTraversalModel};
use crate::model::{
    state::{InputFeature, StateVariableConfig},
    traversal::{
        default::fieldname, TraversalModel, TraversalModelError, TraversalModelService,
    },
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
    ) -> Result<Arc<dyn TraversalModel>, TraversalModelError> {
        let model = GradeTraversalModel::new(self.engine.clone());
        Ok(Arc::new(model))
    }
}
