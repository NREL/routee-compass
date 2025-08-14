use super::GradeConfiguration;
use crate::{
    model::{network::EdgeId, traversal::TraversalModelError},
    util::fs::{read_decoders, read_utils},
};
use kdam::Bar;
use std::sync::Arc;
use uom::{si::f64::Ratio, ConstZero};

pub struct GradeTraversalEngine {
    pub grade_by_edge_id: Option<Arc<Box<[Ratio]>>>,
}

impl GradeTraversalEngine {
    /// builds a grade lookup table from the input file, or, if not provided, stubs a
    /// grade engine that always returns 0.
    pub fn new(config: &GradeConfiguration) -> Result<GradeTraversalEngine, TraversalModelError> {
        let grade_table: Box<[Ratio]> = read_utils::read_raw_file(
            config.grade_input_file.clone(),
            read_decoders::f64,
            Some(Bar::builder().desc("link grades")),
            None,
        )
        .map_err(|e| {
            TraversalModelError::BuildError(format!(
                "failure reading grade table {} due to {}",
                config.grade_input_file.clone(),
                e
            ))
        })?
        .iter()
        .map(|&g| config.grade_unit.to_uom(g))
        .collect::<Vec<Ratio>>()
        .into_boxed_slice();

        let engine = GradeTraversalEngine {
            grade_by_edge_id: Some(Arc::new(grade_table)),
        };

        Ok(engine)
    }

    pub fn get_grade(&self, edge_id: EdgeId) -> Result<Ratio, TraversalModelError> {
        match &self.grade_by_edge_id {
            None => Ok(Ratio::ZERO),
            Some(table) => {
                let grade: &Ratio = table.get(edge_id.as_usize()).ok_or_else(|| {
                    TraversalModelError::TraversalModelFailure(format!(
                        "missing index {edge_id} from grade table"
                    ))
                })?;
                Ok(*grade)
            }
        }
    }
}
