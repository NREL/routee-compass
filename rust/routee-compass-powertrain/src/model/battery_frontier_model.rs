use routee_compass_core::{
    algorithm::search::{Direction, SearchTreeBranch},
    model::{
        frontier::{FrontierModel, FrontierModelError, FrontierModelService},
        network::{Edge, VertexId},
        state::{StateModel, StateVariable},
    },
};
use uom::si::f64::Ratio;

use std::sync::Arc;

use crate::model::fieldname;

#[derive(Clone)]
pub struct BatteryRestriction {
    soc_lower_bound: Ratio,
}

impl FrontierModel for BatteryRestriction {
    fn valid_frontier(
        &self,
        _edge: &Edge,
        state: &[StateVariable],
        _tree: &std::collections::HashMap<VertexId, SearchTreeBranch>,
        _direction: &Direction,
        state_model: &StateModel,
    ) -> Result<bool, FrontierModelError> {
        let soc: Ratio = state_model.get_ratio(state, fieldname::TRIP_SOC).map_err(|_| {
            FrontierModelError::FrontierModelError(
                "BatteryRestriction fronteir model requires the state variable 'soc' but not found".to_string(),
            )
        })?;
        Ok(soc <= self.soc_lower_bound)
    }

    fn valid_edge(&self, _edge: &Edge) -> Result<bool, FrontierModelError> {
        Ok(true)
    }
}

impl Default for BatteryRestriction {
    fn default() -> Self {
        BatteryRestriction {
            soc_lower_bound: Ratio::new::<uom::si::ratio::percent>(1.0),
        }
    }
}

impl FrontierModelService for BatteryRestriction {
    fn build(
        &self,
        _query: &serde_json::Value,
        _state_model: Arc<StateModel>,
    ) -> Result<Arc<dyn FrontierModel>, FrontierModelError> {
        // see if the query has an soc_lower_bound_percent or use the default otherwise
        let soc_lower_bound_percent = _query
            .get("soc_lower_bound_percent")
            .and_then(|v| v.as_f64());
        let model = match soc_lower_bound_percent {
            Some(percent) => BatteryRestriction {
                soc_lower_bound: Ratio::new::<uom::si::ratio::percent>(percent),
            },
            None => BatteryRestriction::default(),
        };
        Ok(Arc::new(model))
    }
}
