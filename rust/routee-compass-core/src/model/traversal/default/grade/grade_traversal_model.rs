use super::GradeTraversalEngine;
use crate::model::{
    network::{Edge, EdgeId, Vertex},
    state::{InputFeature, OutputFeature, StateModel, StateVariable},
    traversal::{TraversalModel, TraversalModelError},
    unit::Grade,
};
use std::sync::Arc;

pub struct GradeTraversalModel {
    pub engine: Arc<GradeTraversalEngine>,
}

impl GradeTraversalModel {
    pub fn new(engine: Arc<GradeTraversalEngine>) -> GradeTraversalModel {
        GradeTraversalModel { engine }
    }
}

impl TraversalModel for GradeTraversalModel {
    /// no upstream state dependencies
    fn input_features(&self) -> Vec<(String, InputFeature)> {
        vec![]
    }

    //
    fn output_features(&self) -> Vec<(String, OutputFeature)> {
        vec![(
            String::from(super::EDGE_GRADE),
            OutputFeature::Grade {
                grade_unit: self.engine.grade_unit.clone(),
                initial: Grade::ZERO,
            },
        )]
    }

    fn traverse_edge(
        &self,
        trajectory: (&Vertex, &Edge, &Vertex),
        state: &mut Vec<StateVariable>,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        let (_, edge, _) = trajectory;
        let grade = self.engine.get_grade(edge.edge_id)?;
        state_model.set_grade(state, super::EDGE_GRADE, &grade, &self.engine.grade_unit)?;
        Ok(())
    }

    fn estimate_traversal(
        &self,
        _od: (&Vertex, &Vertex),
        _state: &mut Vec<StateVariable>,
        _state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        // would be nice if we could use vertex elevation to estimate overall grade change..
        Ok(())
    }
}
