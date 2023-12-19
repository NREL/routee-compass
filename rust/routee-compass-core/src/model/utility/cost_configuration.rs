use super::vehicle::vehicle_cost_mapping::VehicleCostMapping;

pub struct CostConfiguration {
    distance_cost: Option<VehicleCostMapping>,
    time_cost: Option<VehicleCostMapping>,
    electric_cost: Option<VehicleCostMapping>,
    gas_cost: Option<VehicleCostMapping>,
    diesel_cost: Option<VehicleCostMapping>,
}
