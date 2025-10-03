use uom::si::f64::Time;

use super::edge_heading::EdgeHeading;
use super::turn::Turn;
use super::turn_delay_model::TurnDelayModel;
use crate::model::network::EdgeId;
use crate::model::traversal::TraversalModelError;

pub struct TurnDelayTraversalModelEngine {
    pub edge_headings: Box<[EdgeHeading]>,
    pub turn_delay_model: TurnDelayModel,
}

impl TurnDelayTraversalModelEngine {
    pub fn get_delay(&self, prev: EdgeId, next: EdgeId) -> Result<Time, TraversalModelError> {
        let src_heading = get_headings(&self.edge_headings, prev)?;
        let dst_heading = get_headings(&self.edge_headings, next)?;
        let angle = src_heading.bearing_to_destination(&dst_heading);
        match &self.turn_delay_model {
            TurnDelayModel::TabularDiscrete { table } => {
                let turn = Turn::from_angle(angle)?;
                let delay = table.get(&turn).ok_or_else(|| {
                    TraversalModelError::TraversalModelFailure(format!(
                        "table missing entry for turn {turn}"
                    ))
                })?;
                Ok(*delay)
            }
        }
    }
}

/// lookup up the edge heading from the headings table
pub fn get_headings(
    headings_table: &[EdgeHeading],
    edge_id: EdgeId,
) -> Result<EdgeHeading, TraversalModelError> {
    let heading: &EdgeHeading = headings_table.get(edge_id.as_usize()).ok_or_else(|| {
        TraversalModelError::TraversalModelFailure(format!("missing edge id {edge_id} "))
    })?;
    Ok(*heading)
}
