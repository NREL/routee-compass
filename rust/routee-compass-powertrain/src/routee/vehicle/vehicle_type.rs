use std::sync::Arc;

use routee_compass_core::{
    model::traversal::{
        state::state_variable::StateVar, traversal_model_error::TraversalModelError,
    },
    util::unit::{Distance, DistanceUnit, Energy, EnergyUnit, Grade, GradeUnit, Speed, SpeedUnit},
};

use super::VehicleEnergyResult;

pub type VehicleState = Vec<StateVar>;

/// A Vehicle Type represents a class of vehicles with a specific operating model.
pub trait VehicleType: Send + Sync {
    /// Return the name of the vehicle type
    fn name(&self) -> String;

    /// Return the energy required to travel a certain distance at a certain speed and grade.
    ///
    /// Arguments:
    /// * `speed` - The speed at which the vehicle is traveling
    /// * `grade` - The grade of the road
    /// * `distance` - The distance traveled
    /// * `state` - The state of the vehicle
    ///
    /// Returns:
    /// * `VehicleEnergyResult` - The energy required
    fn consume_energy(
        &self,
        speed: (Speed, SpeedUnit),
        grade: (Grade, GradeUnit),
        distance: (Distance, DistanceUnit),
        state: &[StateVar],
    ) -> Result<VehicleEnergyResult, TraversalModelError>;

    /// Return the best case scenario for traveling a certain distance.
    /// This is used in the a-star algorithm as a distance heuristic.
    ///
    /// Arguments:
    /// * `distance` - The distance traveled
    ///
    /// Returns:
    /// * `Energy` - The 'best case' energy required to travel the distance
    fn best_case_energy(
        &self,
        distance: (Distance, DistanceUnit),
    ) -> Result<(Energy, EnergyUnit), TraversalModelError>;

    /// Return the initial state of the vehicle
    fn initial_state(&self) -> VehicleState;

    /// Serialize the state of the vehicle into JSON
    ///
    /// Arguments:
    /// * `state` - The state of the vehicle
    ///
    /// Returns:
    /// * `serde_json::Value` - The serialized state
    fn serialize_state(&self, state: &[StateVar]) -> serde_json::Value;

    /// Serialize any supplemental state information (like units) into JSON
    ///
    /// Arguments:
    /// * `state` - The state of the vehicle
    ///
    /// Returns:
    /// * `serde_json::Value` - The serialized state information
    fn serialize_state_info(&self, state: &[StateVar]) -> serde_json::Value;

    /// Give the vehicle a chance to update itself from the incoming query
    ///
    /// Arguments:
    /// * `query` - The incoming query
    ///
    /// Returns:
    /// * `Arc<dyn VehicleType>` - The updated vehicle type
    fn update_from_query(
        &self,
        query: &serde_json::Value,
    ) -> Result<Arc<dyn VehicleType>, TraversalModelError>;
}
