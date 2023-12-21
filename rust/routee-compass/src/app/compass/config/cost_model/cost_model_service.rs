use crate::app::compass::config::compass_configuration_error::CompassConfigurationError;
use crate::app::compass::config::config_json_extension::ConfigJsonExtensions;
use routee_compass_core::model::cost::{
    cost_aggregation::CostAggregation, cost_model::CostModel,
    network::network_cost_rate::NetworkCostRate, vehicle::vehicle_cost_rate::VehicleCostRate,
};
use std::{collections::HashMap, sync::Arc};

pub struct CostModelService {
    pub vehicle_state_variable_rates: Arc<HashMap<String, VehicleCostRate>>,
    pub network_state_variable_rates: Arc<HashMap<String, NetworkCostRate>>,
    pub state_variable_coefficients: Arc<HashMap<String, f64>>,
    pub default_cost_aggregation: CostAggregation,
}

impl CostModelService {
    /// create a new instance of a utility model service using the provided
    /// values deserialized from app configuration.
    ///
    /// if no default vehicle state variable names are provided, fall back to "distance"
    /// defaults as defined here in this module.
    pub fn new(
        vehicle_state_variable_rates: Option<HashMap<String, VehicleCostRate>>,
        network_state_variable_rates: Option<HashMap<String, NetworkCostRate>>,
        default_state_variable_coefficients: Option<HashMap<String, f64>>,
        default_cost_aggregation: Option<CostAggregation>,
    ) -> Result<CostModelService, CompassConfigurationError> {
        let vm = vehicle_state_variable_rates
            .unwrap_or(CostModelService::default_vehicle_state_variable_rates());
        // let vm = Arc::new(vehicle_mapping);
        let nm = network_state_variable_rates.unwrap_or(HashMap::new());
        let dsvc = match default_state_variable_coefficients {
            Some(coefficients) => {
                if coefficients.is_empty() {
                    Err(CompassConfigurationError::UserConfigurationError(
                        String::from("default vehicle state_variable_coefficients cannot be empty"),
                    ))
                } else {
                    Ok(coefficients)
                }
            }
            None => {
                log::warn!("using default vehicle state variable ['distance']");
                Ok(CostModelService::default_state_variable_coefficients())
            }
        }?;
        let dca = match default_cost_aggregation {
            Some(agg) => agg,
            None => {
                log::warn!("using default cost aggregation 'sum'");
                CostAggregation::Sum
            }
        };
        Ok(CostModelService {
            vehicle_state_variable_rates: Arc::new(vm),
            network_state_variable_rates: Arc::new(nm),
            state_variable_coefficients: Arc::new(dsvc),
            default_cost_aggregation: dca,
        })
    }

    pub fn default_vehicle_state_variable_rates() -> HashMap<String, VehicleCostRate> {
        HashMap::from([
            (String::from("distance"), VehicleCostRate::Raw),
            (String::from("time"), VehicleCostRate::Raw),
            (String::from("energy"), VehicleCostRate::Raw),
            (String::from("energy_liquid"), VehicleCostRate::Raw),
            (String::from("energy_electric"), VehicleCostRate::Raw),
        ])
    }

    pub fn default_state_variable_coefficients() -> HashMap<String, f64> {
        HashMap::from([(String::from("distance"), 1.0)])
    }

    /// a default cost model interprets raw distance values for costs
    pub fn default_cost_model() -> CostModelService {
        log::warn!("using default utility model");
        CostModelService {
            vehicle_state_variable_rates: Arc::new(
                CostModelService::default_vehicle_state_variable_rates(),
            ),
            network_state_variable_rates: Arc::new(HashMap::new()),
            state_variable_coefficients: Arc::new(
                CostModelService::default_state_variable_coefficients(),
            ),
            default_cost_aggregation: CostAggregation::Sum,
        }
    }

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
    /// * `state_variable_names` - list of names describing each slot in the state vector
    ///                            used by the instantiated traversal model.
    ///
    /// # Result
    ///
    /// A CostModel instance to use within a search or an error
    pub fn build(
        &self,
        query: &serde_json::Value,
        traversal_state_variable_names: &[String],
    ) -> Result<CostModel, CompassConfigurationError> {
        let state_variable_coefficients: Arc<HashMap<String, f64>> = query
            .get_config_serde_optional::<HashMap<String, f64>>(
                &"state_variable_coefficients",
                &"utility_model",
            )?
            .map(Arc::new)
            .unwrap_or(self.state_variable_coefficients.clone());

        let state_variable_indices = traversal_state_variable_names
            .iter()
            .enumerate()
            .filter(|(_idx, n)| state_variable_coefficients.contains_key(*n))
            .map(|(idx, n)| (n.clone(), idx))
            .collect::<Vec<_>>();

        let cost_aggregation: CostAggregation = query
            .get_config_serde_optional(&"cost_aggregation", &"utility_model")?
            .unwrap_or(self.default_cost_aggregation.to_owned());

        let model = CostModel::new(
            state_variable_indices,
            state_variable_coefficients,
            self.vehicle_state_variable_rates.clone(),
            self.network_state_variable_rates.clone(),
            cost_aggregation,
        );

        Ok(model)
    }
}
