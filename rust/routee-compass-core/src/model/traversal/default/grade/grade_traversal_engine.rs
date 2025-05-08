use super::GradeConfiguration;
use crate::{
    model::{
        network::EdgeId,
        traversal::TraversalModelError,
        unit::{Grade, GradeUnit},
    },
    util::fs::{read_decoders, read_utils},
};
use kdam::Bar;
use std::sync::Arc;

pub struct GradeTraversalEngine {
    pub grade_by_edge_id: Option<Arc<Box<[Grade]>>>,
    pub grade_unit: GradeUnit,
}

impl GradeTraversalEngine {
    pub fn new(config: &GradeConfiguration) -> Result<GradeTraversalEngine, TraversalModelError> {
        match &config.grade_input_file {
            None => Ok(Self {
                grade_by_edge_id: None,
                grade_unit: config.grade_unit,
            }),
            Some(grade_input_file) => {
                let grade_table: Box<[Grade]> = read_utils::read_raw_file(
                    grade_input_file,
                    read_decoders::default,
                    Some(Bar::builder().desc("link grades")),
                    None,
                )
                .map_err(|e| {
                    TraversalModelError::BuildError(format!(
                        "failure reading grade table {} due to {}",
                        grade_input_file.clone(),
                        e
                    ))
                })?;

                let engine = GradeTraversalEngine {
                    grade_by_edge_id: Some(Arc::new(grade_table)),
                    grade_unit: config.grade_unit,
                };

                Ok(engine)
            }
        }
    }

    pub fn get_grade(&self, edge_id: EdgeId) -> Result<Grade, TraversalModelError> {
        match &self.grade_by_edge_id {
            None => return Ok(Grade::ZERO),
            Some(table) => {
                let grade: &Grade = table.get(edge_id.as_usize()).ok_or_else(|| {
                    TraversalModelError::TraversalModelFailure(format!(
                        "missing index {} from grade table",
                        edge_id
                    ))
                })?;
                Ok(*grade)
            }
        }
    }
}
