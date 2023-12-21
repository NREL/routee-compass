use crate::app::compass::config::compass_configuration_error::CompassConfigurationError;
use crate::app::compass::config::config_json_extension::ConfigJsonExtensions;
use routee_compass_core::model::cost::{
    cost_aggregation::CostAggregation, cost_model::CostModel,
    network::network_cost_mapping::NetworkUtilityMapping,
    vehicle::vehicle_cost_mapping::VehicleUtilityMapping,
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

pub struct CostModelService {
    pub vehicle_mapping: Arc<HashMap<String, VehicleUtilityMapping>>,
    pub network_mapping: Arc<HashMap<String, NetworkUtilityMapping>>,
    pub default_vehicle_dimensions: HashSet<String>,
    pub default_cost_aggregation: CostAggregation,
}

impl CostModelService {
    /// create a new instance of a utility model service using the provided
    /// values deserialized from app configuration.
    ///
    /// if no default vehicle dimensions are provided, fall back to "distance".
    pub fn new(
        vehicle_mapping: Option<HashMap<String, VehicleUtilityMapping>>,
        network_mapping: Option<HashMap<String, NetworkUtilityMapping>>,
        default_vehicle_dimensions: Option<HashSet<String>>,
        default_cost_aggregation: Option<CostAggregation>,
    ) -> Result<CostModelService, CompassConfigurationError> {
        let vm = vehicle_mapping.unwrap_or(CostModelService::default_vehicle_mapping());
        // let vm = Arc::new(vehicle_mapping);
        let nm = network_mapping.unwrap_or(HashMap::new());
        let dvd = match default_vehicle_dimensions {
            Some(dims) => {
                if dims.is_empty() {
                    Err(CompassConfigurationError::UserConfigurationError(
                        String::from("default vehicle utility dimensions cannot be empty"),
                    ))
                } else {
                    Ok(dims)
                }
            }
            None => {
                log::warn!("using default vehicle dimensions ['distance']");
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
            default_vehicle_dimensions: dvd,
            default_cost_aggregation: dca,
        })
    }

    pub fn default_vehicle_mapping() -> HashMap<String, VehicleUtilityMapping> {
        HashMap::from([
            (String::from("distance"), VehicleUtilityMapping::Raw),
            (String::from("time"), VehicleUtilityMapping::Raw),
            (String::from("energy"), VehicleUtilityMapping::Raw),
            (String::from("energy_liquid"), VehicleUtilityMapping::Raw),
            (String::from("energy_electric"), VehicleUtilityMapping::Raw),
        ])
    }

    /// a default cost model interprets raw distance values for costs
    pub fn default_cost_model() -> CostModelService {
        log::warn!("using default utility model");
        CostModelService {
            vehicle_mapping: Arc::new(CostModelService::default_vehicle_mapping()),
            network_mapping: Arc::new(HashMap::new()),
            default_vehicle_dimensions: HashSet::from([String::from("time")]),
            default_cost_aggregation: CostAggregation::Sum,
        }
    }

    /// builds a CostModel based on the incoming query parameters along with the
    /// state dimension names of the traversal model.
    ///
    /// the query is expected to contain the following keys:
    ///
    /// ```python
    /// {
    ///   "vehicle_dimensions": [],  # list of state dimensions to convert to costs
    ///   "cost_aggregation": ''     # operation for combining costs, 'sum' or 'mul'
    /// }
    /// ```
    ///
    /// by providing vehicle dimensions for cost function
    ///
    /// # Arguments
    ///
    /// * `query` - search query
    /// * `state_dimensions` - list of names describing each slot in the state vector
    ///                        used by the instantiated traversal model.
    ///
    /// # Result
    ///
    /// A CostModel instance to use within a search or an error
    pub fn build(
        &self,
        query: &serde_json::Value,
        state_dimensions: &[String],
    ) -> Result<CostModel, CompassConfigurationError> {
        let dimension_names: HashSet<String> = query
            .get_config_serde_optional(&"vehicle_dimensions", &"utility_model")?
            .unwrap_or(self.default_vehicle_dimensions.to_owned());

        let dimensions = state_dimensions
            .iter()
            .enumerate()
            .filter(|(_idx, n)| dimension_names.contains(*n))
            .map(|(idx, n)| (n.clone(), idx))
            .collect::<Vec<_>>();

        let cost_aggregation: CostAggregation = query
            .get_config_serde_optional(&"cost_aggregation", &"utility_model")?
            .unwrap_or(self.default_cost_aggregation.to_owned());

        let model = CostModel::new(
            dimensions,
            self.vehicle_mapping.clone(),
            self.network_mapping.clone(),
            cost_aggregation,
        );

        Ok(model)
    }
}
