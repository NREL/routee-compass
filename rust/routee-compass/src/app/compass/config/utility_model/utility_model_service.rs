use crate::app::compass::config::compass_configuration_error::CompassConfigurationError;
use crate::app::compass::config::config_json_extension::ConfigJsonExtensions;
use routee_compass_core::model::utility::{
    cost_aggregation::CostAggregation, utility_model::UtilityModel,
    vehicle::vehicle_utility_mapping::VehicleUtilityMapping,
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

pub struct UtilityModelService {
    vehicle_mapping: Arc<HashMap<String, VehicleUtilityMapping>>,
}

impl UtilityModelService {
    /// builds a UtilityModel based on the incoming query parameters along with the
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
    /// # Arguments
    ///
    /// * `query` - search query
    /// * `state_dimensions` - list of names describing each slot in the state vector
    ///                        used by the instantiated traversal model.
    ///
    /// # Result
    ///
    /// A UtilityModel instance to use within a search or an error
    pub fn build(
        &self,
        query: serde_json::Value,
        state_dimensions: &[String],
    ) -> Result<UtilityModel, CompassConfigurationError> {
        let dimension_names: HashSet<String> =
            query.get_config_serde(&"vehicle_dimensions", &"utility_model")?;

        let dimensions = state_dimensions
            .iter()
            .enumerate()
            .filter(|(_idx, n)| dimension_names.contains(*n))
            .map(|(idx, n)| (n.clone(), idx))
            .collect::<Vec<_>>();

        let cost_aggregation: CostAggregation =
            query.get_config_serde(&"cost_aggregation", &"utility_model")?;

        let model = UtilityModel::new(dimensions, self.vehicle_mapping.clone(), cost_aggregation);

        Ok(model)
    }
}
