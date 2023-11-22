use routee_compass_core::{
    model::{road_network::edge_id::EdgeId, traversal::traversal_model_error::TraversalModelError},
    util::unit::Grade,
};

pub const ZERO_ENERGY: f64 = 1e-9;

/// look up the grade from the grade table
pub fn get_grade(
    grade_table: &Option<Box<[Grade]>>,
    edge_id: EdgeId,
) -> Result<Grade, TraversalModelError> {
    match grade_table {
        None => Ok(Grade::ZERO),
        Some(gt) => {
            let grade: &Grade = gt.get(edge_id.as_usize()).ok_or(
                TraversalModelError::MissingIdInTabularCostFunction(
                    format!("{}", edge_id),
                    String::from("EdgeId"),
                    String::from("grade table"),
                ),
            )?;
            Ok(*grade)
        }
    }
}
