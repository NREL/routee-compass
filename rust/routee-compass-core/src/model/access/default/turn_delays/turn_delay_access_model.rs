use super::{
    edge_heading::EdgeHeading, turn_delay_access_model_engine::TurnDelayAccessModelEngine,
};
use crate::model::{
    access::{access_model::AccessModel, access_model_error::AccessModelError},
    property::{edge::Edge, vertex::Vertex},
    road_network::edge_id::EdgeId,
    state::{state_feature::StateFeature, state_model::StateModel},
    traversal::state::state_variable::StateVar,
};
use std::sync::Arc;

pub struct TurnDelayAccessModel {
    pub engine: Arc<TurnDelayAccessModelEngine>,
}

impl AccessModel for TurnDelayAccessModel {
    fn access_edge(
        &self,
        traversal: (&Vertex, &Edge, &Vertex, &Edge, &Vertex),
        state: &mut Vec<StateVar>,
        state_model: &StateModel,
    ) -> Result<(), AccessModelError> {
        let (_v1, src, _v2, dst, _v3) = traversal;
        let src_heading = get_headings(&self.engine.edge_headings, src.edge_id)?;
        let dst_heading = get_headings(&self.engine.edge_headings, dst.edge_id)?;
        let angle = src_heading.bearing_to_destination(&dst_heading);
        let (delay, delay_unit) = self.engine.turn_delay_model.get_delay(angle)?;
        state_model.add_time(state, &self.engine.time_feature_name, &delay, delay_unit)?;
        Ok(())
    }

    fn state_features(&self) -> Vec<(String, StateFeature)> {
        vec![]
    }
}

/// lookup up the edge heading from the headings table
pub fn get_headings(
    headings_table: &[EdgeHeading],
    edge_id: EdgeId,
) -> Result<EdgeHeading, AccessModelError> {
    let heading: &EdgeHeading =
        headings_table
            .get(edge_id.as_usize())
            .ok_or_else(|| AccessModelError::RuntimeError {
                name: String::from("turn delay access model"),
                error: format!("missing edge id {} ", edge_id),
            })?;
    Ok(*heading)
}
