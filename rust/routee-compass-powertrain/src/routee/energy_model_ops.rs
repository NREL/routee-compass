use routee_compass_core::model::{
    access::default::turn_delays::EdgeHeading, network::edge_id::EdgeId,
    traversal::TraversalModelError, unit::Grade,
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
                TraversalModelError::TraversalModelFailure(format!(
                    "missing index {} from grade table",
                    edge_id
                ))
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
        TraversalModelError::TraversalModelFailure(format!(
            "missing index {} from headings table",
            edge_id
        ))
    })?;
    Ok(*heading)
}
