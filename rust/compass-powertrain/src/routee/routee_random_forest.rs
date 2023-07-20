use compass_core::model::{
    cost::cost::Cost,
    traversal::{
        function::function::EdgeCostFunction,
        state::{search_state::StateVector, state_variable::StateVar},
    },
};

pub fn build_routee_random_forest() -> EdgeCostFunction {
    // load the random forest model here, similar to route prototype
    // copy that code over from prototype into this module instead of
    // referencing code directly from the compass_prototype lib

    let f: EdgeCostFunction = Box::new(move |o, e, d, s| {
        //
        // lookup routee cost, return energy cost here (instead of Cost::ZERO)
        //
        let energy_cost: Cost = Cost::ZERO;
        let energy_cost_f64: f64 = energy_cost.into_f64();
        let mut updated_state: StateVector = s.to_vec();
        updated_state[0] = updated_state[0] + StateVar(energy_cost_f64);
        Ok((energy_cost, updated_state))
    });

    return f;
}

/// starting state for a RouteE random forest search
pub fn initial_energy_state() -> StateVector {
    vec![StateVar(0.0)]
}
