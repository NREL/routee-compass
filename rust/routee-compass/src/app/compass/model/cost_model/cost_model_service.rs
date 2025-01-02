use crate::app::compass::CompassConfigurationError;
use crate::app::compass::ConfigJsonExtensions;
use routee_compass_core::model::{
    cost::{network::NetworkCostRate, CostAggregation, CostModel, VehicleCostRate},
    state::StateModel,
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

pub struct CostModelService {
    pub vehicle_rates: Arc<HashMap<String, VehicleCostRate>>,
    pub network_rates: Arc<HashMap<String, NetworkCostRate>>,
    pub weights: Arc<HashMap<String, f64>>,
    pub cost_aggregation: CostAggregation,
    pub ignore_unknown_weights: bool,
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
        let weights: Arc<HashMap<String, f64>> = query
            .get_config_serde_optional::<HashMap<String, f64>>(&"weights", &"cost_model")?
            .map(Arc::new)
            .unwrap_or(self.weights.clone());

        // // union the requested state variables with those in the existing traversal model
        // // load only indices that appear in coefficients object
        let state_indices = state_model.to_vec();
        let query_state_indices = state_indices
            .iter()
            .filter(|(n, _idx)| weights.contains_key(n))
            .map(|(n, idx)| (n.clone(), idx))
            .collect::<Vec<_>>();

        // validate user input, no query state variables provided that are unknown to traversal model
        if weights.len() != query_state_indices.len() && !self.ignore_unknown_weights {
            let names_lookup: HashSet<&String> =
                query_state_indices.iter().map(|(n, _)| n).collect();

            let extras = weights
                .clone()
                .keys()
                .filter(|n| !names_lookup.contains(n))
                .cloned()
                .collect::<Vec<_>>()
                .join(",");

            let msg = format!("unknown weights in query: [{}]", extras);
            return Err(CompassConfigurationError::UserConfigurationError(msg));
        }

        // the user can append/replace rates from the query
        let vehicle_rates = query
            .get_config_serde_optional::<HashMap<String, VehicleCostRate>>(
                &"vehicle_rates",
                &"cost_model",
            )
            .map(|opt_rates| match opt_rates {
                Some(rates) => Arc::new(rates),
                None => self.vehicle_rates.clone(),
            })?;

        let cost_aggregation: CostAggregation = query
            .get_config_serde_optional(&"cost_aggregation", &"cost_model")?
            .unwrap_or(self.cost_aggregation.to_owned());

        let model = CostModel::new(
            weights,
            vehicle_rates,
            self.network_rates.clone(),
            cost_aggregation,
            state_model,
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
