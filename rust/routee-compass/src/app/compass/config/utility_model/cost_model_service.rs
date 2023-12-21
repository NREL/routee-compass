use crate::app::compass::config::compass_configuration_error::CompassConfigurationError;
use crate::app::compass::config::config_json_extension::ConfigJsonExtensions;
use routee_compass_core::model::cost::{
    cost_aggregation::CostAggregation, cost_model::CostModel,
    network::network_cost_mapping::NetworkCostMapping,
    vehicle::vehicle_cost_mapping::VehicleCostMapping,
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

pub struct CostModelService {
    pub vehicle_mapping: Arc<HashMap<String, VehicleCostMapping>>,
    pub network_mapping: Arc<HashMap<String, NetworkCostMapping>>,
    pub default_state_variable_names: HashSet<String>,
    pub default_cost_aggregation: CostAggregation,
}

impl CostModelService {
    /// create a new instance of a utility model service using the provided
    /// values deserialized from app configuration.
    ///
    /// if no default vehicle state variable names are provided, fall back to "distance".
    pub fn new(
        vehicle_mapping: Option<HashMap<String, VehicleCostMapping>>,
        network_mapping: Option<HashMap<String, NetworkCostMapping>>,
        default_state_variable_names: Option<HashSet<String>>,
        default_cost_aggregation: Option<CostAggregation>,
    ) -> Result<CostModelService, CompassConfigurationError> {
        let vm = vehicle_mapping.unwrap_or(CostModelService::default_vehicle_mapping());
        // let vm = Arc::new(vehicle_mapping);
        let nm = network_mapping.unwrap_or(HashMap::new());
        let dsvn = match default_state_variable_names {
            Some(dims) => {
                if dims.is_empty() {
                    Err(CompassConfigurationError::UserConfigurationError(
                        String::from("default vehicle cost state_variable_names cannot be empty"),
                    ))
                } else {
                    Ok(dims)
                }
            }
            None => {
                log::warn!("using default vehicle state variable ['distance']");
                Ok(HashSet::from([String::from("distance")]))
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
            vehicle_mapping: Arc::new(vm),
            network_mapping: Arc::new(nm),
            default_state_variable_names: dsvn,
            default_cost_aggregation: dca,
        })
    }

    pub fn default_vehicle_mapping() -> HashMap<String, VehicleCostMapping> {
        HashMap::from([
            (String::from("distance"), VehicleCostMapping::Raw),
            (String::from("time"), VehicleCostMapping::Raw),
            (String::from("energy"), VehicleCostMapping::Raw),
            (String::from("energy_liquid"), VehicleCostMapping::Raw),
            (String::from("energy_electric"), VehicleCostMapping::Raw),
        ])
    }

    /// a default cost model interprets raw distance values for costs
    pub fn default_cost_model() -> CostModelService {
        log::warn!("using default utility model");
        CostModelService {
            vehicle_mapping: Arc::new(CostModelService::default_vehicle_mapping()),
            network_mapping: Arc::new(HashMap::new()),
            default_state_variable_names: HashSet::from([String::from("time")]),
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
        let state_variable_names: HashSet<String> = query
            .get_config_serde_optional(&"state_variable_names", &"utility_model")?
            .unwrap_or(self.default_state_variable_names.to_owned());

        let state_variables = traversal_state_variable_names
            .iter()
            .enumerate()
            .filter(|(_idx, n)| state_variable_names.contains(*n))
            .map(|(idx, n)| (n.clone(), idx))
            .collect::<Vec<_>>();

        let cost_aggregation: CostAggregation = query
            .get_config_serde_optional(&"cost_aggregation", &"utility_model")?
            .unwrap_or(self.default_cost_aggregation.to_owned());

        let model = CostModel::new(
            state_variables,
            self.vehicle_mapping.clone(),
            self.network_mapping.clone(),
            cost_aggregation,
        );

        Ok(model)
    }
}
