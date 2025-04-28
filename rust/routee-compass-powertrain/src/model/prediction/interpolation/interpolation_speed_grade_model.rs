use super::utils::linspace;
use crate::model::prediction::{
    load_prediction_model, model_type::ModelType, prediction_model::PredictionModel,
};
use ninterp::prelude::*;
use routee_compass_core::{
    model::traversal::TraversalModelError,
    model::unit::{
        AsF64, Distance, EnergyRate, EnergyRateUnit, Grade, GradeUnit, Speed, SpeedUnit,
    },
};
use std::path::Path;

pub struct InterpolationSpeedGradeModel {
    interpolator: Interp2DOwned<f64, strategy::Linear>,
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
        let speed_value = speed_unit.convert(&speed, &self.speed_unit).as_f64();
        let grade_value = grade_unit.convert(&grade, &self.grade_unit).as_f64();

        // snap incoming speed and grade to the grid
        let min_speed = *self.interpolator.data.grid.get(0).map(|s| s.first()).flatten().ok_or_else(|| {
            TraversalModelError::TraversalModelFailure(
                "Could not get first speed value from powertrain interpolation result; are x-values empty?".to_string(),
            )
        })?;
        let max_speed = *self.interpolator.data.grid.get(0).map(|s| s.last()).flatten().ok_or_else(|| {
            TraversalModelError::TraversalModelFailure(
                "Could not get last speed value from powertrain interpolation result; are x-values empty?".to_string(),
            )
        })?;
        let min_grade = *self.interpolator.data.grid.get(1).map(|g| g.first()).flatten().ok_or_else(|| {
            TraversalModelError::TraversalModelFailure(
                "Could not get first grade value from powertrain interpolation result; are y-values empty?".to_string(),
            )
        })?;
        let max_grade = *self.interpolator.data.grid.get(1).map(|g| g.last()).flatten().ok_or_else(|| {
            TraversalModelError::TraversalModelFailure(
                "Could not get last grade value from powertrain interpolation result; are y-values empty?".to_string(),
            )
        })?;

        let speed_value = speed_value.clamp(min_speed, max_speed);
        let grade_value = grade_value.clamp(min_grade, max_grade);

        let y = self
            .interpolator
            .interpolate(&[speed_value, grade_value])
            .map_err(|e| {
                TraversalModelError::TraversalModelFailure(format!(
                    "Failed to interpolate speed/grade model output during prediction: {}",
                    e
                ))
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
        let speed_values = ndarray::Array1::from_vec(linspace(
            speed_bounds.0.as_f64(),
            speed_bounds.1.as_f64(),
            speed_bins,
        ));
        let grade_values = ndarray::Array1::from_vec(linspace(
            grade_bounds.0.as_f64(),
            grade_bounds.1.as_f64(),
            grade_bins,
        ));

        // Use a unit distance so we can get the energy per unit distance
        let distance = Distance::new(1.0);
        let distance_unit = energy_rate_unit.associated_distance_unit();

        // Predict energy rate values across the whole grid
        let mut values = ndarray::Array2::zeros((speed_bins, grade_bins));
        for i in 0..speed_bins {
            for j in 0..grade_bins {
                let (energy, _energy_unit) = model.predict(
                    (Speed::new(speed_values[i]), speed_unit),
                    (Grade::new(grade_values[j]), grade_unit),
                    (distance, distance_unit),
                )?;
                values[(i, j)] = energy.as_f64();
            }
        }

        let interpolator = Interp2D::new(
            speed_values,
            grade_values,
            values,
            strategy::Linear,
            Extrapolate::Error,
        )
        .map_err(|e| {
            TraversalModelError::TraversalModelFailure(format!(
                "Failed to validate interpolation model: {}",
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
    use crate::model::prediction::prediction_model::PredictionModel;
    use routee_compass_core::model::unit::EnergyRateUnit;

    #[test]
    fn test_interpolation_speed_grade_model() {
        let model_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("model")
            .join("test")
            .join("Toyota_Camry.bin");

        let interp_model = InterpolationSpeedGradeModel::new(
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

        let underlying_model = load_prediction_model(
            "Toyota Camry".to_string(),
            &model_path,
            ModelType::Smartcore,
            SpeedUnit::MilesPerHour,
            GradeUnit::Decimal,
            EnergyRateUnit::GallonsGasolinePerMile,
            None,
            None,
            None,
        )
        .unwrap();

        // let's check to make sure the interpolation model is
        // producing similar results to the underlying model

        for speed in 0..100 {
            for grade in -20..20 {
                let (interp_energy_rate, _energy_rate_unit) = interp_model
                    .predict(
                        (Speed::new(speed as f64), SpeedUnit::MilesPerHour),
                        (Grade::new(grade as f64), GradeUnit::Percent),
                    )
                    .unwrap();
                let (underlying_energy_rate, _energy_rate_unit) = underlying_model
                    .prediction_model
                    .predict(
                        (Speed::new(speed as f64), SpeedUnit::MilesPerHour),
                        (Grade::new(grade as f64), GradeUnit::Percent),
                    )
                    .unwrap();

                // check if they're within 1% of each other
                let diff = (interp_energy_rate.as_f64() - underlying_energy_rate.as_f64())
                    / underlying_energy_rate.as_f64();
                assert!(diff.abs() < 0.01);
            }
        }

        let (energy_rate, energy_rate_unit) = interp_model
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
