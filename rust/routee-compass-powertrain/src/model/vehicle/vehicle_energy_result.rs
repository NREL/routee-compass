use routee_compass_core::model::{
    state::StateVariable,
    unit::{Energy, EnergyUnit},
};

pub struct VehicleEnergyResult {
    pub energy: Energy,
    pub energy_unit: EnergyUnit,
    pub updated_state: Vec<StateVariable>,
}
