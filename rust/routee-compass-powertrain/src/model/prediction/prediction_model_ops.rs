use super::{
    interpolation::InterpolationModel, model_type::ModelType, smartcore::SmartcoreModel,
    PredictionModel, PredictionModelRecord,
};
use itertools::Itertools;
use routee_compass_core::model::{
    state::InputFeature,
    traversal::TraversalModelError,
    unit::{
        AsF64, Convert, EnergyRate, EnergyRateUnit, Grade, GradeUnit, Speed, SpeedUnit, UnitError,
    },
};
use std::{borrow::Cow, path::Path, sync::Arc};

pub fn load_prediction_model<P: AsRef<Path>>(
    name: String,
    model_path: &P,
    model_type: ModelType,
    input_features: Vec<(String, InputFeature)>,
    energy_rate_unit: EnergyRateUnit,
    ideal_energy_rate_option: Option<EnergyRate>,
    real_world_energy_adjustment_option: Option<f64>,
) -> Result<PredictionModelRecord, TraversalModelError> {
    let prediction_model: Arc<dyn PredictionModel> = match model_type.clone() {
        ModelType::Smartcore => {
            let model = SmartcoreModel::new(model_path, input_features.clone(), energy_rate_unit)?;
            Arc::new(model)
        }
        ModelType::Interpolate {
            underlying_model_type: underlying_model,
            feature_bounds,
        } => {
            let model = InterpolationModel::new(
                model_path,
                *underlying_model,
                name.clone(),
                input_features.clone(),
                feature_bounds.clone(),
                energy_rate_unit,
            )?;
            Arc::new(model)
        }
    };
    let ideal_energy_rate = match ideal_energy_rate_option {
        None => find_min_energy_rate(&prediction_model, &input_features, &energy_rate_unit)?,
        Some(ier) => ier,
    };

    let real_world_energy_adjustment = real_world_energy_adjustment_option.unwrap_or(1.0);

    Ok(PredictionModelRecord {
        name,
        prediction_model,
        model_type,
        input_features,
        energy_rate_unit,
        ideal_energy_rate,
        real_world_energy_adjustment,
    })
}

pub fn transpose<T>(v: Vec<Vec<T>>) -> Result<Vec<Vec<T>>, TraversalModelError> {
    assert!(!v.is_empty());
    if v.iter().any(|n| n.is_empty()) {
        return Err(TraversalModelError::BuildError(
            "cannot transpose an empty vector".to_string(),
        ));
    }
    let len = v[0].len();
    let mut iters: Vec<_> = v.into_iter().map(|n| n.into_iter()).collect();
    let result = (0..len)
        .map(|_| {
            iters
                .iter_mut()
                .map(|n| n.next().unwrap())
                .collect::<Vec<T>>()
        })
        .collect();
    Ok(result)
}

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

    let mut feature_vectors: Vec<Vec<f64>> = Vec::new();

    for (_, input_feature) in input_features {
        let input_vector: Vec<f64> = match input_feature {
            InputFeature::Speed(unit) => {
                match unit {
                    None => {
                        return Err(TraversalModelError::TraversalModelFailure(format!(
                            "{} expected input feature: {} to have an associated unit in the model prediction",
                            MIN_ENERGY_ERROR_MESSAGE,
                            input_feature
                        )))
                    },
                    Some(unit) => {
                        // create a vector of speed values
                        get_speed_sample_values(unit)?.into_iter().map(|s| s.as_f64()).collect()
                    }
                }
            }
            InputFeature::Grade(unit) => {
                match unit {
                    None => {
                        return Err(TraversalModelError::TraversalModelFailure(format!(
                            "{} expected input feature: {} to have an associated unit in the model prediction",
                            MIN_ENERGY_ERROR_MESSAGE,
                            input_feature
                        )))
                    },
                    Some(unit) => {
                        // create a vector of grade values
                        get_grade_sample_values(unit)?.into_iter().map(|g| g.as_f64()).collect()
                    }
                }
            }
            _ => {
                return Err(TraversalModelError::TraversalModelFailure(format!(
                    "{} got an unexpected input feature in the smartcore model prediction {}",
                    MIN_ENERGY_ERROR_MESSAGE, input_feature
                )))
            }
        };
        feature_vectors.push(input_vector);
    }

    let transposed_vectors = transpose(feature_vectors)?;
    for feature_vec in transposed_vectors {
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
        "found minimum energy: {}/{} in {} milliseconds",
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
