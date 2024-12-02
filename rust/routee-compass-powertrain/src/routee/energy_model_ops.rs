use routee_compass_core::model::{
    access::default::turn_delays::edge_heading::EdgeHeading, graph::edge_id::EdgeId,
    traversal::traversal_model_error::TraversalModelError, unit::Grade,
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
            let grade: &Grade = gt.get(edge_id.as_usize()).ok_or_else(|| {
                TraversalModelError::MissingIdInTabularCostFunction(
                    format!("{}", edge_id),
                    String::from("EdgeId"),
                    String::from("grade table"),
                )
            })?;
            Ok(*grade)
        }
    }
}

/// lookup up the edge heading from the headings table
pub fn get_headings(
    headings_table: &[EdgeHeading],
    edge_id: EdgeId,
) -> Result<EdgeHeading, TraversalModelError> {
    let heading: &EdgeHeading = headings_table.get(edge_id.as_usize()).ok_or_else(|| {
        TraversalModelError::MissingIdInTabularCostFunction(
            format!("{}", edge_id),
            String::from("EdgeId"),
            String::from("headings table"),
        )
    })?;
    Ok(*heading)
}
