use super::{cost_ops, network::NetworkCostRate, CostAggregation, VehicleCostRate};
use crate::model::cost::CostModelError;
use crate::model::network::Edge;
use crate::model::state::StateModel;
use crate::model::state::StateVariable;
use crate::model::unit::Cost;
use indexmap::IndexMap;
use itertools::Itertools;
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

#[derive(Serialize, Clone, Debug)]
pub struct TraversalCost {
    pub total_cost: Cost,
    pub components: HashMap<String, Cost>,
}

/// implementation of a model for calculating Cost from a state transition.
/// vectorized, where each index in these vectors matches the corresponding index
/// in the state model.
pub struct CostModel {
    features: IndexMap<String, CostFeature>,
    displayed_costs: HashSet<String>,
    weights_mapping: Arc<HashMap<String, f64>>,
    vehicle_rate_mapping: Arc<HashMap<String, VehicleCostRate>>,
    cost_aggregation: CostAggregation,
}

#[derive(Default, Clone, Debug)]
pub struct CostFeature {
    pub weight: f64,
    pub vehicle_cost_rate: VehicleCostRate,
    pub network_cost_rate: NetworkCostRate,
    pub display: bool,
}

impl CostFeature {
    /// builds a cost feature instance based on the potential combinations of optional configuration arguments
    pub fn new(
        weight: Option<&f64>,
        vehicle_rate: Option<&VehicleCostRate>,
        network_rate: Option<&NetworkCostRate>,
        display: bool,
    ) -> CostFeature {
        match (weight, vehicle_rate, network_rate) {
            (None, _, _) => CostFeature::default(),
            (Some(_), None, None) => CostFeature::default(),
            (Some(w), None, Some(n)) => CostFeature {
                weight: *w,
                vehicle_cost_rate: VehicleCostRate::default(),
                network_cost_rate: n.clone(),
                display,
            },
            (Some(w), Some(v), None) => CostFeature {
                weight: *w,
                vehicle_cost_rate: v.clone(),
                network_cost_rate: NetworkCostRate::default(),
                display,
            },
            (Some(w), Some(v), Some(n)) => CostFeature {
                weight: *w,
                vehicle_cost_rate: v.clone(),
                network_cost_rate: n.clone(),
                display,
            },
        }
    }
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
    /// * `displayed_costs`      - on serialization, the cost values to calculate
    /// * `weights`              - user-provided weighting factors for each feature
    /// * `vehicle_rate_mapping` - for each feature name, a vehicle cost rate for that feature
    /// * `network_rate_mapping` - for each feature name, a network cost rate for that feature
    /// * `cost_aggregation`     - function for aggregating each feature cost (for example, Sum)
    /// * `state_model`          - state model instance for this search
    pub fn new(
        displayed_costs: HashSet<String>,
        weights_mapping: Arc<HashMap<String, f64>>,
        vehicle_rate_mapping: Arc<HashMap<String, VehicleCostRate>>,
        network_rate_mapping: Arc<HashMap<String, NetworkCostRate>>,
        cost_aggregation: CostAggregation,
        state_model: Arc<StateModel>,
        log_description: bool,
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
            if log_description {
                let desc = cost_ops::describe_cost_feature_configuration(
                    name,
                    &displayed_costs,
                    weights_mapping.clone(),
                    vehicle_rate_mapping.clone(),
                );
                log::info!("{desc}");
            }
            let w_opt = weights_mapping.get(name);
            let v_opt = vehicle_rate_mapping.get(name);
            let n_opt = network_rate_mapping.get(name);
            let feature = CostFeature::new(w_opt, v_opt, n_opt, displayed_costs.contains(name));

            total_weight += feature.weight;
            features.insert(name.clone(), feature);
        }

        if total_weight == 0.0 {
            // TODO: update this Error variant after refactor
            return Err(CostModelError::InvalidCostVariables(vec![]));
        }
        Ok(CostModel {
            features,
            displayed_costs,
            weights_mapping,
            vehicle_rate_mapping,
            cost_aggregation,
        })
    }

    /// calculates the total trip cost of traversing the provided edge.
    pub fn traversal_cost_2(
        &self,
        edge: &Edge,
        state: &[StateVariable],
        state_model: Arc<StateModel>,
    ) -> Result<TraversalCost, CostModelError> {
        todo!()
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
            result.insert(
                name.clone(),
                json![{
                    Self::WEIGHT: json![feature.weight],
                    Self::VEHICLE_RATE: json![feature.vehicle_cost_rate],
                    Self::NETWORK_RATE: json![feature.network_cost_rate],
                    Self::INDEX: json![index]
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
    const FEATURE: &'static str = "feature";
    const WEIGHT: &'static str = "weight";
    const COST_AGGREGATION: &'static str = "cost_aggregation";
}
