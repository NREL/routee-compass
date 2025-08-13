use super::{feature_bounds::FeatureBounds, utils::linspace};
use crate::model::prediction::{
    model_type::ModelType, prediction_model::PredictionModel, smartcore::SmartcoreModel,
};
use itertools::Itertools;
use ndarray::{ArrayD, IxDyn};
use ninterp::prelude::*;
use routee_compass_core::model::{
    state::InputFeature, traversal::TraversalModelError, unit::EnergyRateUnit,
};
use std::{collections::HashMap, path::Path};

pub struct InterpolationModel {
    interpolator: InterpNDOwned<f64, strategy::Linear>,
    energy_rate_unit: EnergyRateUnit,
}

impl PredictionModel for InterpolationModel {
    fn predict(
        &self,
        feature_vector: &[f64],
    ) -> Result<(f64, EnergyRateUnit), TraversalModelError> {
        let y = self.interpolator.interpolate(feature_vector).map_err(|e| {
            TraversalModelError::TraversalModelFailure(format!(
                "Failed to interpolate speed/grade model output during prediction: {e}"
            ))
        })?;

        let energy_rate = y;
        Ok((energy_rate, self.energy_rate_unit))
    }
}

impl InterpolationModel {
    pub fn new<P: AsRef<Path>>(
        underlying_model_path: &P,
        underlying_model_type: ModelType,
        input_features: Vec<InputFeature>,
        feature_bounds: HashMap<String, FeatureBounds>,
        energy_rate_unit: EnergyRateUnit,
    ) -> Result<Self, TraversalModelError> {
        // load underlying model to build the interpolation grid
        let model = match underlying_model_type {
            ModelType::Smartcore => SmartcoreModel::new(underlying_model_path, energy_rate_unit)?,
            _ => {
                return Err(TraversalModelError::TraversalModelFailure(
                    "Got unexpected model type when building the interpolation model".to_string(),
                ))
            }
        };

        let mut grid: Vec<ndarray::Array1<f64>> = Vec::new();

        for input_feature in input_features.iter() {
            let feature_name = input_feature.name();
            let feature_bounds = feature_bounds.get(&feature_name).ok_or_else(|| {
                TraversalModelError::BuildError(format!(
                    "Missing feature bounds for {feature_name}, got: {feature_bounds:?}"
                ))
            })?;

            let feature_grid = ndarray::Array1::from_vec(linspace(
                feature_bounds.lower_bound,
                feature_bounds.upper_bound,
                feature_bounds.num_bins,
            ));
            grid.push(feature_grid);
        }

        let shape: Vec<usize> = grid.iter().map(|feature| feature.len()).collect();
        let mut values = ArrayD::<f64>::zeros(IxDyn(&shape));

        // Predict energy rate values across the whole grid
        let index_ranges: Vec<_> = shape.iter().map(|&len| 0..len).collect();

        for indices in index_ranges.into_iter().multi_cartesian_product() {
            // Get the actual feature values corresponding to the current indices
            let input: Vec<f64> = indices
                .iter()
                .enumerate()
                .map(|(dim, &i)| grid[dim][i])
                .collect();

            // predict the energy rate
            let (energy_rate, _energy_rate_unit) = model.predict(&input).map_err(|e| {
                TraversalModelError::TraversalModelFailure(format!(
                    "Failed to predict energy rate: {e}"
                ))
            })?;
            values[IxDyn(&indices)] = energy_rate;
        }

        let interpolator = InterpND::new(grid, values, strategy::Linear, Extrapolate::Clamp)
            .map_err(|e| {
                TraversalModelError::TraversalModelFailure(format!(
                    "Failed to validate interpolation model: {e}"
                ))
            })?;

        Ok(InterpolationModel {
            interpolator,
            energy_rate_unit,
        })
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::*;
    use crate::model::prediction::prediction_model::PredictionModel;
    use routee_compass_core::model::unit::{EnergyRateUnit, RatioUnit, SpeedUnit};

    #[test]
    fn test_interpolation_speed_grade_model() {
        let model_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("model")
            .join("test")
            .join("Toyota_Camry.bin");

        let input_features = vec![
            InputFeature::Speed {
                name: "speed".to_string(),
                unit: Some(SpeedUnit::MPH),
            },
            InputFeature::Ratio {
                name: "grade".to_string(),
                unit: Some(RatioUnit::Decimal),
            },
        ];
        let feature_bounds = HashMap::from([
            (
                "speed".to_string(),
                FeatureBounds {
                    lower_bound: 1.0,
                    upper_bound: 100.0,
                    num_bins: 100,
                },
            ),
            (
                "grade".to_string(),
                FeatureBounds {
                    lower_bound: -0.2,
                    upper_bound: 0.2,
                    num_bins: 41,
                },
            ),
        ]);
        let interp_model = InterpolationModel::new(
            &model_path,
            ModelType::Smartcore,
            input_features.clone(),
            feature_bounds,
            EnergyRateUnit::GGPM,
        )
        .unwrap();

        let underlying_model = SmartcoreModel::new(&model_path, EnergyRateUnit::GGPM).unwrap();

        // let's check to make sure the interpolation model is
        // producing similar results to the underlying model

        for speed in 0..100 {
            for grade in -20..20 {
                let speed_f64 = speed as f64;
                let grade_f64 = grade as f64 / 100.0;
                let input = vec![speed_f64, grade_f64];
                let (interp_energy_rate, _energy_rate_unit) = interp_model.predict(&input).unwrap();
                let (underlying_energy_rate, _energy_rate_unit) =
                    underlying_model.predict(&input).unwrap();

                // check if they're within 1% of each other
                let diff = (interp_energy_rate - underlying_energy_rate) / underlying_energy_rate;
                assert!(diff.abs() < 0.01);
            }
        }

        let (energy_rate, energy_rate_unit) = interp_model.predict(&[50.0, 0.0]).unwrap();

        assert_eq!(energy_rate_unit, EnergyRateUnit::GGPM);

        // energy rate should be between 28-32 mpg
        let expected_lower = 1.0 / 32.0;
        let expected_upper = 1.0 / 28.0;
        assert!(energy_rate >= expected_lower);
        assert!(energy_rate <= expected_upper);
    }
}
