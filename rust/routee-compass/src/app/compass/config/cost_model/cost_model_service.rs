use crate::app::compass::config::compass_configuration_error::CompassConfigurationError;
use crate::app::compass::config::config_json_extension::ConfigJsonExtensions;
use routee_compass_core::model::{
    cost::{
        cost_aggregation::CostAggregation, cost_model::CostModel,
        network::network_cost_rate::NetworkCostRate, vehicle::vehicle_cost_rate::VehicleCostRate,
    },
    state::state_model::StateModel,
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

pub struct CostModelService {
    pub vehicle_state_variable_rates: Arc<HashMap<String, VehicleCostRate>>,
    pub network_state_variable_rates: Arc<HashMap<String, NetworkCostRate>>,
    pub state_variable_coefficients: Arc<HashMap<String, f64>>,
    pub cost_aggregation: CostAggregation,
}

impl CostModelService {
    /// builds a CostModel based on the incoming query parameters along with the
    /// state variable names of the traversal model.
    ///
    /// the query is expected to contain the following keys:
    ///
    /// ```python
    /// {
    ///   "state_variable_names": [],  # list of state variables to convert to costs
    ///   "cost_aggregation": ''     # operation for combining costs, 'sum' or 'mul'
    /// }
    /// ```
    ///
    /// by providing vehicle state attributes for cost function
    ///
    /// # Arguments
    ///
    /// * `query` - search query
    /// * `traversal_state_variable_names` - list of names describing each slot in the state vector
    ///                            used by the instantiated traversal model.
    ///
    /// # Result
    ///
    /// A CostModel instance to use within a search or an error
    pub fn build(
        &self,
        query: &serde_json::Value,
        state_model: Arc<StateModel>,
    ) -> Result<CostModel, CompassConfigurationError> {
        // user-provided coefficients used to prioritize each state variable in the cost model
        // at minimum, we default to the "distance" traveled.
        // invariant: this hashmap dictates the list of keys for all subsequent CostModel hashmaps.
        let state_variable_coefficients: Arc<HashMap<String, f64>> = query
            .get_config_serde_optional::<HashMap<String, f64>>(
                &"state_variable_coefficients",
                &"cost_model",
            )?
            .map(Arc::new)
            .unwrap_or(self.state_variable_coefficients.clone());

        // // union the requested state variables with those in the existing traversal model
        // // load only indices that appear in coefficients object
        let state_indices = state_model.state_model_vec();
        let query_state_indices = state_indices
            .iter()
            .filter(|(n, _idx)| state_variable_coefficients.contains_key(n))
            .map(|(n, idx)| (n.clone(), idx))
            .collect::<Vec<_>>();

        // validate user input, no query state variables provided that are unknown to traversal model
        if state_variable_coefficients.len() != query_state_indices.len() {
            let names_lookup: HashSet<&String> =
                query_state_indices.iter().map(|(n, _)| n).collect();

            let extras = state_variable_coefficients
                .clone()
                .keys()
                .filter(|n| !names_lookup.contains(n))
                .cloned()
                .collect::<Vec<_>>()
                .join(",");

            let msg = format!("unknown state variables in query: [{}]", extras);
            return Err(CompassConfigurationError::UserConfigurationError(msg));
        }

        // the user can append/replace rates from the query
        let vehicle_rates = query
            .get_config_serde_optional::<HashMap<String, VehicleCostRate>>(
                &"vehicle_state_variable_rates",
                &"cost_model",
            )
            .map(|opt_rates| match opt_rates {
                Some(rates) => Arc::new(rates),
                None => self.vehicle_state_variable_rates.clone(),
            })?;

        let cost_aggregation: CostAggregation = query
            .get_config_serde_optional(&"cost_aggregation", &"cost_model")?
            .unwrap_or(self.cost_aggregation.to_owned());

        let model = CostModel::new(
            state_variable_coefficients,
            state_model,
            vehicle_rates,
            self.network_state_variable_rates.clone(),
            cost_aggregation,
        )
        .map_err(|e| {
            CompassConfigurationError::UserConfigurationError(format!(
                "failed to build cost model: {}",
                e
            ))
        })?;

        Ok(model)
    }
}
