use super::edge_heading::EdgeHeading;
use super::turn_delay_model::TurnDelayModel;

pub struct TurnDelayAccessModelEngine {
    pub edge_headings: Box<[EdgeHeading]>,
    pub turn_delay_model: TurnDelayModel,
    pub time_feature_name: String,
}
