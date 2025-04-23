use super::{ElevationChange, GradeTraversalEngine};
use crate::model::{
    network::{Edge, EdgeId, Vertex},
    state::{StateFeature, StateModel, StateVariable},
    traversal::{TraversalModel, TraversalModelError},
    unit::{DistanceUnit, Grade},
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
    fn input_features(&self) -> Vec<(String, StateFeature)> {
        vec![]
    }

    fn output_features(&self) -> Vec<(String, StateFeature)> {
        todo!()
    }

    fn traverse_edge(
        &self,
        trajectory: (&Vertex, &Edge, &Vertex),
        state: &mut Vec<StateVariable>,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        let (_, edge, _) = trajectory;
        let grade = get_grade(self.engine.grade_by_edge_id.clone(), edge.edge_id)?;
        state_model.set_grade(state, super::LEG_GRADE, &grade, &self.engine.grade_unit)?;
        let distance = state_model.get_distance(state, super::LEG_DISTANCE, &DistanceUnit::Feet)?;
        let elevation_change = ElevationChange::new(
            (&distance, &DistanceUnit::Feet),
            (&grade, &self.engine.grade_unit),
            &self.engine.elevation_unit,
        )?;
        elevation_change.add_elevation_to_state(state, state_model)?;
        Ok(())
    }

    fn estimate_traversal(
        &self,
        _od: (&Vertex, &Vertex),
        state: &mut Vec<StateVariable>,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        // would be nice if we could use vertex elevation to estimate overall grade change..
        state_model.set_grade(
            state,
            super::LEG_GRADE,
            &Grade::ZERO,
            &self.engine.grade_unit,
        )?;
        Ok(())
    }
}

pub fn get_grade(
    grade_table: Arc<Box<[Grade]>>,
    edge_id: EdgeId,
) -> Result<Grade, TraversalModelError> {
    let grade: &Grade = grade_table.get(edge_id.as_usize()).ok_or_else(|| {
        TraversalModelError::TraversalModelFailure(format!(
            "missing index {} from grade table",
            edge_id
        ))
    })?;
    Ok(*grade)
}
