use routee_compass_core::model::{
    state::{StateFeature, StateModel, StateVariable},
    traversal::TraversalModelError,
    unit::{Distance, DistanceUnit, Energy, EnergyUnit, Grade, GradeUnit, Speed, SpeedUnit},
};
use std::sync::Arc;

/// A Vehicle Type represents a class of vehicles with a specific operating model.
pub trait VehicleType: Send + Sync {
    /// Return the name of the vehicle type
    fn name(&self) -> String;

    /// lists the state variables expected by this vehicle type. these are
    /// appended to the base state model set at configuration time.
    fn state_features(&self) -> Vec<(String, StateFeature)>;

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
        state: &mut Vec<StateVariable>,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError>;

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

    /// Return the best case scenario for traveling a certain distance.
    /// This is used in the a-star algorithm as a distance heuristic.
    ///
    /// Arguments:
    /// * `distance` - The distance traveled
    ///
    /// Returns:
    /// * `Energy` - The 'best case' energy required to travel the distance
    fn best_case_energy_state(
        &self,
        distance: (Distance, DistanceUnit),
        state: &mut Vec<StateVariable>,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError>;

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
