use routee_compass_core::{
    model::traversal::{
        state::state_variable::StateVar, traversal_model_error::TraversalModelError,
    },
    util::unit::{Distance, DistanceUnit, Energy, EnergyUnit, Grade, GradeUnit, Speed, SpeedUnit},
};

pub type VehicleState = Vec<StateVar>;

pub struct VehicleEnergyResult {
    pub energy: Energy,
    pub energy_unit: EnergyUnit,
    pub updated_state: VehicleState,
}

pub trait Vehicle: Send + Sync {
    fn predict_energy(
        &self,
        speed: (Speed, SpeedUnit),
        grade: (Grade, GradeUnit),
        distance: (Distance, DistanceUnit),
        state: &[StateVar],
    ) -> Result<VehicleEnergyResult, TraversalModelError>;

    /// Return the best case scenario for traveling a certain distance
    fn best_case_energy(
        &self,
        distance: (Distance, DistanceUnit),
    ) -> Result<(Energy, EnergyUnit), TraversalModelError>;

    /// Return the initial state of the vehicle
    fn initial_state(&self) -> VehicleState;

    fn serialize_state(&self, state: &VehicleState) -> serde_json::Value;
    fn serialize_state_info(&self, state: &VehicleState) -> serde_json::Value;
}
