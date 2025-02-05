use crate::model::state::StateFeature;

use super::{DistanceUnit, EnergyRateUnit, EnergyUnit, GradeUnit, SpeedUnit, TimeUnit, WeightUnit};

pub trait HasUnits {
    /// lists the state variables expected by this model that are not
    /// defined on the base configuration. for example, if this model
    /// has state variables that differ based on the query, they can be injected
    /// into the model by listing them here.
    fn state_features(&self) -> &Vec<(String, StateFeature)>;

    fn get_distance_unit(&self) -> Option<DistanceUnit>;

    fn get_energy_unit(&self) -> Option<EnergyUnit>;

    fn get_energy_rate_unit(&self) -> Option<EnergyRateUnit>;

    fn get_grade_unit(&self) -> Option<GradeUnit>;

    fn get_speed_unit(&self) -> Option<SpeedUnit>;

    fn get_time_unit(&self) -> Option<TimeUnit>;

    fn get_weight_unit(&self) -> Option<WeightUnit>;
}
