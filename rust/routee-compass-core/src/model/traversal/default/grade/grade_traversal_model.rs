use uom::{si::f64::Ratio, ConstZero};

use super::GradeTraversalEngine;
use crate::{
    algorithm::search::SearchTree,
    model::{
        network::{Edge, Vertex},
        state::{InputFeature, StateModel, StateVariable, StateVariableConfig},
        traversal::{default::fieldname, TraversalModel, TraversalModelError},
        unit::RatioUnit,
    },
};
use std::sync::Arc;

pub struct GradeTraversalModel {
    pub engine: Arc<GradeTraversalEngine>,
    // Pre-resolved index for performance
    edge_grade_idx: usize,
}

impl GradeTraversalModel {
    pub fn new(engine: Arc<GradeTraversalEngine>, edge_grade_idx: usize) -> GradeTraversalModel {
        GradeTraversalModel {
            engine,
            edge_grade_idx,
        }
    }
}

impl TraversalModel for GradeTraversalModel {
    fn name(&self) -> String {
        String::from("Grade Traversal Model")
    }

    fn traverse_edge(
        &self,
        trajectory: (&Vertex, &Edge, &Vertex),
        state: &mut Vec<StateVariable>,
        _tree: &SearchTree,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        let (_, edge, _) = trajectory;
        let grade = self.engine.get_grade(edge.edge_id)?;

        state_model.set_ratio_by_index(state, self.edge_grade_idx, &grade)?;
        Ok(())
    }

    fn estimate_traversal(
        &self,
        _od: (&Vertex, &Vertex),
        _state: &mut Vec<StateVariable>,
        _tree: &SearchTree,
        _state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        // would be nice if we could use vertex elevation to estimate overall grade change..
        Ok(())
    }
}
