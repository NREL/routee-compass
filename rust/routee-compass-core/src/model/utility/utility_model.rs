use crate::model::road_network::edge_id::EdgeId;
use crate::model::traversal::state::state_variable::StateVar;
use crate::model::utility::cost::Cost;
use crate::model::utility::utility_error::UtilityError;

pub trait UtilityModel: Sync + Send {
    fn traversal_cost(
        &self,
        edge_id: EdgeId,
        prev_state: &[StateVar],
        next_state: &[StateVar],
    ) -> Result<Cost, UtilityError>;

    fn access_cost(
        &self,
        prev_edge_id: Option<EdgeId>,
        next_edge_id: EdgeId,
        prev_state: &[StateVar],
        next_state: &[StateVar],
    ) -> Result<Cost, UtilityError>;
}
