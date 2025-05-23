use super::PredictionModel;
use itertools::Itertools;
use routee_compass_core::model::{
    state::InputFeature,
    traversal::TraversalModelError,
    unit::{
        AsF64, Convert, EnergyRate, EnergyRateUnit, Grade, GradeUnit, Speed, SpeedUnit, UnitError,
    },
};
use std::{borrow::Cow, sync::Arc};

const MIN_ENERGY_ERROR_MESSAGE: &str =
    "Failure while executing grid search for minimum energy rate in prediction model:";

/// sweep speed and grade values to find the minimum energy per mile rate from the incoming rf model
pub fn find_min_energy_rate(
    model: &Arc<dyn PredictionModel>,
    input_features: &[(String, InputFeature)],
    energy_model_energy_rate_unit: &EnergyRateUnit,
) -> Result<EnergyRate, TraversalModelError> {
    // sweep a fixed set of speed and grade values to find the minimum energy per mile rate from the incoming rf model
    let mut minimum_energy_rate = EnergyRate::from(f64::MAX);
    let start_time = std::time::Instant::now();

    // Create vectors of sample values for each feature type
    let mut sample_values: Vec<Vec<f64>> = Vec::new();

    for (_, input_feature) in input_features {
        let values = match input_feature {
            InputFeature::Speed(unit) => get_speed_sample_values(unit)?
                .into_iter()
                .map(|s| s.as_f64())
                .collect(),
            InputFeature::Grade(unit) => get_grade_sample_values(unit)?
                .into_iter()
                .map(|g| g.as_f64())
                .collect(),
            _ => {
                return Err(TraversalModelError::TraversalModelFailure(format!(
                    "{} got an unexpected input feature in the smartcore model prediction {}",
                    MIN_ENERGY_ERROR_MESSAGE, input_feature
                )))
            }
        };
        sample_values.push(values);
    }

    for feature_vec in sample_values.into_iter().multi_cartesian_product() {
        // Predict energy rate
        let (energy_rate, _) = model.predict(&feature_vec).map_err(|e| {
            TraversalModelError::BuildError(format!("{} {}", MIN_ENERGY_ERROR_MESSAGE, e))
        })?;

        if energy_rate < minimum_energy_rate {
            minimum_energy_rate = energy_rate;
        }
    }

    let end_time = std::time::Instant::now();
    let search_time = end_time - start_time;

    log::debug!(
        "found minimum energy: {} {} in {} milliseconds",
        minimum_energy_rate,
        energy_model_energy_rate_unit,
        search_time.as_millis()
    );

    Ok(minimum_energy_rate)
}

/// generate Percent Grade values in the range [-20, 0] converted to the target grade unit
fn get_grade_sample_values(grade_unit: &GradeUnit) -> Result<Vec<Grade>, UnitError> {
    (1..101)
        .map(|i| {
            let grade_dec_f64 = ((i as f64) * 0.2) - 20.0; // values in range [-20.0, 0.0]
            let mut converted = Cow::Owned(Grade::from(grade_dec_f64));
            GradeUnit::Decimal.convert(&mut converted, grade_unit)?;
            Ok(converted.into_owned())
        })
        .try_collect()
}

/// generate MPH Speed values in the range [1, 100] converted to the target speed unit
fn get_speed_sample_values(speed_unit: &SpeedUnit) -> Result<Vec<Speed>, UnitError> {
    (1..1001)
        .map(|i| {
            let mph_f64 = i as f64 * 0.1; // values in range [0.0, 100.0]
            let mut converted = Cow::Owned(Speed::from(mph_f64));
            SpeedUnit::MPS.convert(&mut converted, speed_unit)?;
            Ok(converted.into_owned())
        })
        .try_collect()
}
