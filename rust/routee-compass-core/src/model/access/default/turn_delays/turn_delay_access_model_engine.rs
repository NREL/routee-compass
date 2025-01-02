use super::edge_heading::EdgeHeading;
use super::turn::Turn;
use super::turn_delay_model::TurnDelayModel;
use crate::model::access::AccessModelError;
use crate::model::network::{Edge, EdgeId, Vertex};
use crate::model::unit::{Time, TimeUnit};

pub struct TurnDelayAccessModelEngine {
    pub edge_headings: Box<[EdgeHeading]>,
    pub turn_delay_model: TurnDelayModel,
    pub time_feature_name: String,
}

impl TurnDelayAccessModelEngine {
    pub fn get_delay<'a>(
        &'a self,
        traversal: (&Vertex, &Edge, &Vertex, &Edge, &Vertex),
    ) -> Result<(Time, &'a TimeUnit), AccessModelError> {
        let (_v1, src, _v2, dst, _v3) = traversal;
        let src_heading = get_headings(&self.edge_headings, src.edge_id)?;
        let dst_heading = get_headings(&self.edge_headings, dst.edge_id)?;
        let angle = src_heading.bearing_to_destination(&dst_heading);
        match &self.turn_delay_model {
            TurnDelayModel::TabularDiscrete { table, time_unit } => {
                let turn = Turn::from_angle(angle)?;
                let delay = table.get(&turn).ok_or_else(|| {
                    let name = String::from("tabular discrete turn delay model");
                    let error = format!("table missing entry for turn {}", turn);
                    AccessModelError::RuntimeError { name, error }
                })?;
                Ok((*delay, time_unit))
            } // TurnDelayModel::TabularDiscreteWithRoadClasses { table, time_unit } => {}
        }
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
