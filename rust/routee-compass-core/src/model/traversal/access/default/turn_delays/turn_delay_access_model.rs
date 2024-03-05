use super::edge_heading::EdgeHeading;
use super::turn_delay_model::TurnDelayModel;
use crate::model::road_network::edge_id::EdgeId;
use crate::model::traversal::access::access_model::AccessModel;
use crate::model::{
    property::{edge::Edge, vertex::Vertex},
    traversal::{
        state::traversal_state::TraversalState, traversal_model_error::TraversalModelError,
    },
};

pub struct TurnDelayAccessModel {
    edge_headings: Box<[EdgeHeading]>,
    turn_delay_model: TurnDelayModel,
}

impl TurnDelayAccessModel {}

impl AccessModel for TurnDelayAccessModel {
    fn access_edge(
        &self,
        traversal: (&Vertex, &Edge, &Vertex, &Edge, &Vertex),
        state: &TraversalState,
        state_variable_indices: Vec<(String, usize)>,
    ) -> Result<Option<TraversalState>, TraversalModelError> {
        let (_v1, src, _v2, dst, _v3) = traversal;
        let src_heading = get_headings(&self.edge_headings, src.edge_id)?;
        let dst_heading = get_headings(&self.edge_headings, dst.edge_id)?;
        let angle = src_heading.bearing_to_destination(&dst_heading);

        // TODO:
        // WAIT, how do we get a time unit here?
        let target_time_unit = &crate::model::unit::TimeUnit::Hours;

        let delay = self.turn_delay_model.get_delay(angle, target_time_unit)?;
        let (_, idx) = state_variable_indices
            .iter()
            .find(|(n, _)| n == "time")
            .ok_or_else(|| {
                TraversalModelError::InternalError(String::from(
                    "turn delay model assumes a 'time' state variable which is not present",
                ))
            })?;

        let mut updated_state = state.clone();
        if updated_state.len() <= *idx {
            return Err(TraversalModelError::InternalError(format!(
                "turn delay model expected 'time' variable at index {} which is out of range for state {:?}",
                idx,
                state
            )));
        }

        updated_state[*idx] = updated_state[*idx] + delay.into();
        Ok(Some(updated_state))
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
