use super::{
    cost_ops, network::NetworkCostRate, CostAggregation, CostFeature, TraversalCost,
    VehicleCostRate,
};
use crate::algorithm::search::SearchTree;
use crate::model::cost::CostModelError;
use crate::model::network::Edge;
use crate::model::network::Vertex;
use crate::model::state::StateModel;
use crate::model::state::StateVariable;
use indexmap::IndexMap;
use itertools::Itertools;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

/// implementation of a model for calculating Cost from a state transition.
/// vectorized, where each index in these vectors matches the corresponding index
/// in the state model.
pub struct CostModel {
    features: IndexMap<String, CostFeature>,
    weights_mapping: Arc<HashMap<String, f64>>,
    vehicle_rate_mapping: Arc<HashMap<String, VehicleCostRate>>,
    network_rate_mapping: Arc<HashMap<String, NetworkCostRate>>,
    cost_aggregation: CostAggregation,
}

impl CostModel {
    /// builds a cost model for a specific query.
    ///
    /// this search instance has a state model that dictates the location of each feature.
    /// here we aim to vectorize a mapping from those features into the cost weights,
    /// vehicle cost rates and network cost rates related to that feature.
    /// at runtime, we can iterate through these vectors to compute the cost.
    ///
    /// # Arguments
    /// * `displayed_costs`      - on serialization, the cost values to calculate. state variable names can be called out explicitly here that do not contribute to the total cost.
    /// * `weights`              - user-provided weighting factors for each feature
    /// * `vehicle_rate_mapping` - for each feature name, a vehicle cost rate for that feature
    /// * `network_rate_mapping` - for each feature name, a network cost rate for that feature
    /// * `cost_aggregation`     - function for aggregating each feature cost (for example, Sum)
    /// * `state_model`          - state model instance for this search
    pub fn new(
        weights_mapping: Arc<HashMap<String, f64>>,
        vehicle_rate_mapping: Arc<HashMap<String, VehicleCostRate>>,
        network_rate_mapping: Arc<HashMap<String, NetworkCostRate>>,
        cost_aggregation: CostAggregation,
        state_model: Arc<StateModel>,
    ) -> Result<CostModel, CostModelError> {
        let ignored_weights = weights_mapping
            .keys()
            .filter(|k| !state_model.contains_key(k))
            .collect_vec();
        if !ignored_weights.is_empty() {
            return Err(CostModelError::InvalidWeightNames(
                ignored_weights.iter().map(|k| k.to_string()).collect(),
                state_model.keys().cloned().collect_vec(),
            ));
        }

        let mut features = IndexMap::new();
        let mut total_weight = 0.0;

        for (name, _) in state_model.iter() {
            // always instantiate a value for each vector, diverting to default (zero-valued) if not provided
            // which has the following effect:
            // - weight: deactivates costs for this feature (product)
            // - v_rate: ignores vehicle costs for this feature (sum)
            // - n_rate: ignores network costs for this feature (sum)
            let w_opt = weights_mapping.get(name);
            let v_opt = vehicle_rate_mapping.get(name);
            let n_opt = network_rate_mapping.get(name);
            let feature = CostFeature::new(name.clone(), w_opt, v_opt, n_opt);

            total_weight += feature.weight;
            features.insert(name.clone(), feature);
        }

        if total_weight == 0.0 {
            // TODO: update this Error variant after refactor
            return Err(CostModelError::InvalidCostVariables(vec![]));
        }
        Ok(CostModel {
            features,
            weights_mapping,
            vehicle_rate_mapping,
            network_rate_mapping,
            cost_aggregation,
        })
    }

    /// calculates the total trip cost of traversing the provided edge.
    pub fn traversal_cost(
        &self,
        trajectory: (&Vertex, &Edge, &Vertex),
        state: &[StateVariable],
        tree: &SearchTree,
        state_model: &StateModel,
    ) -> Result<TraversalCost, CostModelError> {
        let mut result = TraversalCost::default();
        for (name, feature) in self.features.iter() {
            let v_cost = feature
                .vehicle_cost_rate
                .compute_cost(name, state, state_model)?;
            let n_cost =
                feature
                    .network_cost_rate
                    .network_cost(trajectory, state, tree, state_model)?;
            let cost = v_cost + n_cost;
            result.insert(name, cost, feature.weight);
        }
        Ok(result)
    }

    /// calculates the total trip cost of traversing the provided edge.
    pub fn estimate_cost(
        &self,
        state: &[StateVariable],
        state_model: &StateModel,
    ) -> Result<TraversalCost, CostModelError> {
        let mut result = TraversalCost::default();
        for (name, feature) in self.features.iter() {
            let v_cost = feature
                .vehicle_cost_rate
                .compute_cost(name, state, state_model)?;
            result.insert(name, v_cost, feature.weight);
        }
        Ok(result)
    }

    /// Serializes other information about a cost model as a JSON value.
    ///
    /// # Arguments
    ///
    /// * `state` - the state to serialize information from
    ///
    /// # Returns
    ///
    /// JSON containing information such as the units (kph, hours, etc) or other
    /// traversal info (charge events, days traveled, etc)
    pub fn serialize_cost_info(&self) -> Result<serde_json::Value, CostModelError> {
        let mut result = serde_json::Map::with_capacity(self.features.len());
        for (index, (name, feature)) in self.features.iter().enumerate() {
            let desc = cost_ops::describe_cost_feature_configuration(
                name,
                self.weights_mapping.clone(),
                self.vehicle_rate_mapping.clone(),
                self.network_rate_mapping.clone(),
            );
            result.insert(
                name.clone(),
                json![{
                    Self::WEIGHT: json![feature.weight],
                    Self::VEHICLE_RATE: json![feature.vehicle_cost_rate],
                    Self::NETWORK_RATE: json![feature.network_cost_rate.rate_type()],
                    Self::INDEX: json![index],
                    Self::DESCRIPTION: json![desc],
                }],
            );
        }

        result.insert(
            Self::COST_AGGREGATION.to_string(),
            json![self.cost_aggregation],
        );

        Ok(json![result])
    }

    const INDEX: &'static str = "index";
    const VEHICLE_RATE: &'static str = "vehicle_rate";
    const NETWORK_RATE: &'static str = "network_rate";
    const WEIGHT: &'static str = "weight";
    const COST_AGGREGATION: &'static str = "cost_aggregation";
    const DESCRIPTION: &'static str = "description";
}
