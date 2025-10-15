use crate::model::cost::{network::NetworkCostRate, VehicleCostRate};

/// configures how cost can be calculated for a state feature.
#[derive(Clone, Debug)]
pub struct CostFeature {
    pub name: String,
    pub weight: f64,
    pub vehicle_cost_rate: VehicleCostRate,
    pub network_cost_rate: NetworkCostRate,
}

impl CostFeature {
    /// creates a zero-valued cost feature for a given feature name
    pub fn zero(name: String) -> CostFeature {
        CostFeature {
            name,
            weight: f64::default(),
            vehicle_cost_rate: VehicleCostRate::default(),
            network_cost_rate: NetworkCostRate::default(),
        }
    }

    /// builds a cost feature instance based on the potential combinations of optional configuration arguments
    pub fn new(
        name: String,
        weight: Option<&f64>,
        vehicle_rate: Option<&VehicleCostRate>,
        network_rate: Option<&NetworkCostRate>,
    ) -> CostFeature {
        match (weight, vehicle_rate, network_rate) {
            (None, _, _) => CostFeature::zero(name),
            (Some(_), None, None) => CostFeature::zero(name),
            (Some(w), None, Some(n)) => CostFeature {
                name,
                weight: *w,
                vehicle_cost_rate: VehicleCostRate::default(),
                network_cost_rate: n.clone(),
            },
            (Some(w), Some(v), None) => CostFeature {
                name,
                weight: *w,
                vehicle_cost_rate: v.clone(),
                network_cost_rate: NetworkCostRate::default(),
            },
            (Some(w), Some(v), Some(n)) => CostFeature {
                name,
                weight: *w,
                vehicle_cost_rate: v.clone(),
                network_cost_rate: n.clone(),
            },
        }
    }
}
