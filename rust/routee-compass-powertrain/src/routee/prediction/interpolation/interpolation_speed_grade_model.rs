use std::path::Path;

use crate::routee::prediction::{
    load_prediction_model, model_type::ModelType, prediction_model::PredictionModel,
};

use routee_compass_core::{
    model::traversal::traversal_model_error::TraversalModelError,
    model::unit::{
        as_f64::AsF64, Distance, EnergyRate, EnergyRateUnit, Grade, GradeUnit, Speed, SpeedUnit,
    },
};

use super::bilinear_interp::BilinearInterp;
use super::utils::linspace;

pub struct InterpolationSpeedGradeModel {
    interpolator: BilinearInterp,
    speed_unit: SpeedUnit,
    grade_unit: GradeUnit,
    energy_rate_unit: EnergyRateUnit,
}

impl PredictionModel for InterpolationSpeedGradeModel {
    fn predict(
        &self,
        speed: (Speed, SpeedUnit),
        grade: (Grade, GradeUnit),
    ) -> Result<(EnergyRate, EnergyRateUnit), TraversalModelError> {
        let (speed, speed_unit) = speed;
        let (grade, grade_unit) = grade;
        let speed_value = speed_unit.convert(speed, self.speed_unit).as_f64();
        let grade_value = grade_unit.convert(grade, self.grade_unit).as_f64();

        // snap incoming speed and grade to the grid
        let min_speed = self.interpolator.x[0].0;
        let max_speed = self.interpolator.x[self.interpolator.x.len() - 1].0;
        let min_grade = self.interpolator.y[0].0;
        let max_grade = self.interpolator.y[self.interpolator.y.len() - 1].0;

        let speed_value = speed_value.max(min_speed).min(max_speed);
        let grade_value = grade_value.max(min_grade).min(max_grade);

        let y = self
            .interpolator
            .interpolate(speed_value, grade_value)
            .map_err(|e| {
                TraversalModelError::PredictionModel(format!("Failed to interpolate: {}", e))
            })?;

        let energy_rate = EnergyRate::new(y);
        Ok((energy_rate, self.energy_rate_unit))
    }
}

impl InterpolationSpeedGradeModel {
    #[allow(clippy::too_many_arguments)]
    pub fn new<P: AsRef<Path>>(
        underlying_model_path: &P,
        underlying_model_type: ModelType,
        underlying_model_name: String,
        speed_unit: SpeedUnit,
        speed_bounds: (Speed, Speed),
        speed_bins: usize,
        grade_unit: GradeUnit,
        grade_bounds: (Grade, Grade),
        grade_bins: usize,
        energy_rate_unit: EnergyRateUnit,
    ) -> Result<Self, TraversalModelError> {
        // load underlying model to build the interpolation grid
        let model = load_prediction_model(
            underlying_model_name,
            underlying_model_path,
            underlying_model_type,
            speed_unit,
            grade_unit,
            energy_rate_unit,
            None,
            None,
            None,
        )?;

        // Create a linear grid of speed and grade values
        let speed_values = linspace(speed_bounds.0.as_f64(), speed_bounds.1.as_f64(), speed_bins);
        let grade_values = linspace(grade_bounds.0.as_f64(), grade_bounds.1.as_f64(), grade_bins);

        // Predict energy rate values across the whole grid
        let mut values = Vec::new();

        // Use a unit distance so we can get the energy per unit distance
        let distance = Distance::new(1.0);
        let distance_unit = energy_rate_unit.associated_distance_unit();

        for speed_value in speed_values.clone().into_iter() {
            let mut row: Vec<f64> = Vec::new();
            for grade_value in grade_values.clone().into_iter() {
                let (energy, _energy_unit) = model
                    .predict(
                        (Speed::new(speed_value), speed_unit),
                        (Grade::new(grade_value), grade_unit),
                        (distance, distance_unit),
                    )
                    .map_err(|e| TraversalModelError::PredictionModel(e.to_string()))?;
                row.push(energy.as_f64());
            }
            values.push(row);
        }

        let interpolator =
            BilinearInterp::new(speed_values, grade_values, values).map_err(|e| {
                TraversalModelError::PredictionModel(format!(
                    "Failed to create interpolation model: {}",
                    e
                ))
            })?;

        Ok(InterpolationSpeedGradeModel {
            interpolator,
            speed_unit,
            grade_unit,
            energy_rate_unit,
        })
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::*;
    use crate::routee::prediction::prediction_model::PredictionModel;
    use routee_compass_core::model::unit::EnergyRateUnit;

    #[test]
    fn test_interpolation_speed_grade_model() {
        let model_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("routee")
            .join("test")
            .join("Toyota_Camry.bin");

        let model = InterpolationSpeedGradeModel::new(
            &model_path,
            ModelType::Smartcore,
            "Toyota Camry".to_string(),
            SpeedUnit::MilesPerHour,
            (Speed::new(0.0), Speed::new(100.0)),
            101,
            GradeUnit::Decimal,
            (Grade::new(-0.20), Grade::new(0.20)),
            41,
            EnergyRateUnit::GallonsGasolinePerMile,
        )
        .unwrap();

        let (energy_rate, energy_rate_unit) = model
            .predict(
                (Speed::new(50.0), SpeedUnit::MilesPerHour),
                (Grade::new(0.0), GradeUnit::Percent),
            )
            .unwrap();

        assert_eq!(energy_rate_unit, EnergyRateUnit::GallonsGasolinePerMile);

        // energy rate should be between 28-32 mpg
        let expected_lower = EnergyRate::new(1.0 / 32.0);
        let expected_upper = EnergyRate::new(1.0 / 28.0);
        assert!(energy_rate >= expected_lower);
        assert!(energy_rate <= expected_upper);
    }
}
