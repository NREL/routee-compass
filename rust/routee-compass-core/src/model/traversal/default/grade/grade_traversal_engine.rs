use super::GradeConfiguration;
use crate::{
    model::{
        traversal::TraversalModelError,
        unit::{DistanceUnit, Grade, GradeUnit},
    },
    util::fs::{read_decoders, read_utils},
};
use kdam::Bar;
use std::sync::Arc;

pub struct GradeTraversalEngine {
    pub grade_by_edge_id: Arc<Box<[Grade]>>,
    pub grade_unit: GradeUnit,
    pub elevation_unit: DistanceUnit,
}

impl GradeTraversalEngine {
    pub fn new(config: &GradeConfiguration) -> Result<GradeTraversalEngine, TraversalModelError> {
        let grade_table: Box<[Grade]> = read_utils::read_raw_file(
            &config.grade_input_file,
            read_decoders::default,
            Some(Bar::builder().desc("link grades")),
            None,
        )
        .map_err(|e| {
            TraversalModelError::BuildError(format!(
                "failure reading grade table {} due to {}",
                config.grade_input_file.clone(),
                e
            ))
        })?;

        let engine = GradeTraversalEngine {
            grade_by_edge_id: Arc::new(grade_table),
            grade_unit: config.grade_unit,
            elevation_unit: config.elevation_unit.unwrap_or(DistanceUnit::Feet),
        };

        Ok(engine)
    }
}
