use std::sync::Arc;

use routee_compass_core::{
    model::traversal::TraversalModelError,
    model::unit::{
        AsF64, Distance, DistanceUnit, Energy, EnergyRate, EnergyRateUnit, EnergyUnit, Grade,
        GradeUnit, Speed, SpeedUnit,
    },
    util::cache_policy::float_cache_policy::FloatCachePolicy,
};

use super::{model_type::ModelType, PredictionModel};
/// A struct to hold the prediction model and associated metadata
pub struct PredictionModelRecord {
    pub name: String,
    pub prediction_model: Arc<dyn PredictionModel>,
    pub model_type: ModelType,
    pub speed_unit: SpeedUnit,
    pub grade_unit: GradeUnit,
    pub energy_rate_unit: EnergyRateUnit,
    pub ideal_energy_rate: EnergyRate,
    pub real_world_energy_adjustment: f64,
    pub cache: Option<FloatCachePolicy>,
}

impl PredictionModelRecord {
    pub fn predict(
        &self,
        speed: (Speed, SpeedUnit),
        grade: (Grade, GradeUnit),
        distance: (Distance, DistanceUnit),
    ) -> Result<(Energy, EnergyUnit), TraversalModelError> {
        let (speed, speed_unit) = speed;
        let (distance, distance_unit) = distance;
        let (grade, grade_unit) = grade;

        let energy_rate = match &self.cache {
            Some(cache) => {
                let key = vec![speed.as_f64(), grade.as_f64()];
                match cache.get(&key)? {
                    Some(er) => EnergyRate::from(er),
                    None => {
                        let (energy_rate, _energy_rate_unit) = self
                            .prediction_model
                            .predict((speed, speed_unit), (grade, grade_unit))?;
                        cache.update(&key, energy_rate.as_f64())?;
                        energy_rate
                    }
                }
            }
            None => {
                let (energy_rate, _energy_rate_unit) = self
                    .prediction_model
                    .predict((speed, speed_unit), (grade, grade_unit))?;
                energy_rate
            }
        };

        let energy_rate_real_world = energy_rate * self.real_world_energy_adjustment;

        let (energy, energy_unit) = Energy::create(
            (&distance, &distance_unit),
            (&energy_rate_real_world, &self.energy_rate_unit),
        )?;

        Ok((energy, energy_unit))
    }
}
