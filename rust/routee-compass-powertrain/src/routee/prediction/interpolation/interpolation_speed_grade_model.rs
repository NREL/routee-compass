use std::path::Path;

use crate::routee::prediction::prediction_model::PredictionModel;

use routee_compass_core::{
    model::traversal::traversal_model_error::TraversalModelError,
    model::unit::{as_f64::AsF64, EnergyRate, EnergyRateUnit, Grade, GradeUnit, Speed, SpeedUnit},
};
use smartcore::{
    ensemble::random_forest_regressor::RandomForestRegressor, linalg::basic::matrix::DenseMatrix,
};

use super::utils::{linspace, BilinearInterp};

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
        routee_model_path: &P,
        speed_unit: SpeedUnit,
        speed_bounds: (Speed, Speed),
        speed_bins: usize,
        grade_unit: GradeUnit,
        grade_bounds: (Grade, Grade),
        grade_bins: usize,
        energy_rate_unit: EnergyRateUnit,
    ) -> Result<Self, TraversalModelError> {
        // Load random forest binary file
        let rf_binary = std::fs::read(routee_model_path).map_err(|e| {
            TraversalModelError::FileReadError(
                routee_model_path.as_ref().to_path_buf(),
                e.to_string(),
            )
        })?;
        // Load the random forest regressor so we can exercise it on a linear grid
        let rf: RandomForestRegressor<f64, f64, DenseMatrix<f64>, Vec<f64>> =
            bincode::deserialize(&rf_binary).map_err(|e| {
                TraversalModelError::FileReadError(
                    routee_model_path.as_ref().to_path_buf(),
                    e.to_string(),
                )
            })?;

        // Create a linear grid of speed and grade values
        let speed_values = linspace(speed_bounds.0.as_f64(), speed_bounds.1.as_f64(), speed_bins);
        let grade_values = linspace(grade_bounds.0.as_f64(), grade_bounds.1.as_f64(), grade_bins);

        // Predict energy rate values across the whole grid
        let mut values = Vec::new();
        for speed_value in speed_values.clone().into_iter() {
            let mut row: Vec<f64> = Vec::new();
            for grade_value in grade_values.clone().into_iter() {
                let x = DenseMatrix::from_2d_vec(&vec![vec![speed_value, grade_value]]);
                let y = rf
                    .predict(&x)
                    .map_err(|e| TraversalModelError::PredictionModel(e.to_string()))?;
                row.push(y[0]);
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
