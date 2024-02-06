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
    time_fieldname: String,
}

impl TurnDelayAccessModel {}

impl AccessModel for TurnDelayAccessModel {
    fn access_edge(
        &self,
        _v1: &Vertex,
        src: &Edge,
        _v2: &Vertex,
        dst: &Edge,
        _v3: &Vertex,
        state: &TraversalState,
    ) -> Result<Option<TraversalState>, TraversalModelError> {
        let src_heading = get_headings(&self.edge_headings, src.edge_id)?;
        let dst_heading = get_headings(&self.edge_headings, dst.edge_id)?;
        let angle = src_heading.next_edge_angle(&dst_heading);

        // TODO:
        // - an AccessModel should be able to update the state
        //   - turn delays (time)
        //   - charging stations (energy)
        // - perhaps there should be a broker that is upstream of the access and traversal
        //   models that manages state allocation/cloning, inspection and updates

        // let delay = self.turn_delay_model.get_delay(angle, target_time_unit)?;
        // let updated_state = add_time_to_state(state, time);
        // Ok(Some(updated_state))
        todo!()
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
