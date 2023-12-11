use std::{
    num::NonZeroUsize,
    sync::{Arc, Mutex},
};

use lru::LruCache;
use routee_compass_core::{
    model::traversal::traversal_model_error::TraversalModelError,
    util::unit::{
        as_f64::AsF64, Distance, DistanceUnit, Energy, EnergyRate, EnergyRateUnit, EnergyUnit,
        Grade, GradeUnit, Speed, SpeedUnit,
    },
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
    cache: Mutex<LruCache<(i32, i32), EnergyRate>>,
}

impl PredictionModelRecord {
    pub fn new(
        name: String,
        prediction_model: Arc<dyn PredictionModel>,
        model_type: ModelType,
        speed_unit: SpeedUnit,
        grade_unit: GradeUnit,
        energy_rate_unit: EnergyRateUnit,
        ideal_energy_rate: EnergyRate,
        real_world_energy_adjustment: f64,
        max_cache_size: Option<usize>,
    ) -> Result<Self, TraversalModelError> {
        let max_cache_size = NonZeroUsize::new(max_cache_size.unwrap_or(10000)).ok_or(
            TraversalModelError::BuildError(
                "maximum_cache_size must be greater than 0".to_string(),
            ),
        )?;
        let cache = LruCache::new(max_cache_size);
        Ok(Self {
            name,
            prediction_model,
            model_type,
            speed_unit,
            grade_unit,
            energy_rate_unit,
            ideal_energy_rate,
            real_world_energy_adjustment,
            cache: Mutex::new(cache),
        })
    }
    pub fn predict(
        &self,
        speed: (Speed, SpeedUnit),
        grade: (Grade, GradeUnit),
        distance: (Distance, DistanceUnit),
    ) -> Result<(Energy, EnergyUnit), TraversalModelError> {
        let (distance, distance_unit) = distance;
        // convert speed to kph and then round to nearest integer
        let speed_kph_int = speed
            .1
            .convert(speed.0, SpeedUnit::KilometersPerHour)
            .as_f64()
            .round() as i32;
        let grade_millis_int = grade.1.convert(grade.0, GradeUnit::Millis).as_f64().round() as i32;

        let mut cache = self.cache.lock().unwrap();
        let energy_rate = match cache.get(&(speed_kph_int, grade_millis_int)) {
            Some(er) => *er,
            None => {
                let (energy_rate, _energy_rate_unit) =
                    self.prediction_model.predict(speed, grade)?;
                energy_rate
            }
        };
        cache.put((speed_kph_int, grade_millis_int), energy_rate);
        std::mem::drop(cache);

        let energy_rate_real_world = energy_rate * self.real_world_energy_adjustment;

        let (energy, energy_unit) = Energy::create(
            energy_rate_real_world,
            self.energy_rate_unit,
            distance,
            distance_unit,
        )?;

        Ok((energy, energy_unit))
    }
}
