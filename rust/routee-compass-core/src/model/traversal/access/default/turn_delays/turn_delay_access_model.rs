use super::edge_heading::EdgeHeading;
use crate::model::road_network::edge_id::EdgeId;
use crate::model::traversal::traversal_model_error::TraversalModelError;

pub struct TurnDelayAccessModel {
    edge_headings: Box<[EdgeHeading]>,
    time_fieldname: String,
}

impl TurnDelayAccessModel {}

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
