use super::{
    interpolation::InterpolationSpeedGradeModel, model_type::ModelType,
    smartcore::SmartcoreSpeedGradeModel, PredictionModel, PredictionModelRecord,
};
use itertools::Itertools;
use routee_compass_core::{
    model::{
        traversal::TraversalModelError,
        unit::{
            Convert, EnergyRate, EnergyRateUnit, Grade, GradeUnit, Speed, SpeedUnit, UnitError,
        },
    },
    util::cache_policy::float_cache_policy::FloatCachePolicy,
};
use std::{borrow::Cow, path::Path, sync::Arc};

#[cfg(feature = "onnx")]
use crate::model::prediction::onnx::OnnxSpeedGradeModel;

#[allow(clippy::too_many_arguments)]
pub fn load_prediction_model<P: AsRef<Path>>(
    name: String,
    model_path: &P,
    model_type: ModelType,
    speed_unit: SpeedUnit,
    grade_unit: GradeUnit,
    energy_rate_unit: EnergyRateUnit,
    ideal_energy_rate_option: Option<EnergyRate>,
    real_world_energy_adjustment_option: Option<f64>,
    cache: Option<FloatCachePolicy>,
) -> Result<PredictionModelRecord, TraversalModelError> {
    let prediction_model: Arc<dyn PredictionModel> = match model_type.clone() {
        ModelType::Smartcore => {
            let model = SmartcoreSpeedGradeModel::new(
                model_path,
                speed_unit,
                grade_unit,
                energy_rate_unit,
            )?;
            Arc::new(model)
        }
        ModelType::Onnx => {
            #[cfg(feature = "onnx")]
            {
                let model =
                    OnnxSpeedGradeModel::new(model_path, speed_unit, grade_unit, energy_rate_unit)?;
                Arc::new(model)
            }
            #[cfg(not(feature = "onnx"))]
            {
                return Err(TraversalModelError::BuildError(
                    "Cannot build Onnx model without `onnx` feature enabled for compass-powertrain"
                        .to_string(),
                ));
            }
        }
        ModelType::Interpolate {
            underlying_model_type: underlying_model,
            speed_lower_bound,
            speed_upper_bound,
            speed_bins: speed_bin_size,
            grade_lower_bound,
            grade_upper_bound,
            grade_bins: grade_bin_size,
        } => {
            let model = InterpolationSpeedGradeModel::new(
                model_path,
                *underlying_model,
                name.clone(),
                speed_unit,
                (speed_lower_bound, speed_upper_bound),
                speed_bin_size,
                grade_unit,
                (grade_lower_bound, grade_upper_bound),
                grade_bin_size,
                energy_rate_unit,
            )?;
            Arc::new(model)
        }
    };
    let ideal_energy_rate = match ideal_energy_rate_option {
        None => find_min_energy_rate(&prediction_model, speed_unit, grade_unit, &energy_rate_unit)?,
        Some(ier) => ier,
    };

    let real_world_energy_adjustment = real_world_energy_adjustment_option.unwrap_or(1.0);

    Ok(PredictionModelRecord {
        name,
        prediction_model,
        model_type,
        speed_unit,
        grade_unit,
        energy_rate_unit,
        ideal_energy_rate,
        real_world_energy_adjustment,
        cache,
    })
}

/// sweep speed and grade values to find the minimum energy per mile rate from the incoming rf model
pub fn find_min_energy_rate(
    model: &Arc<dyn PredictionModel>,
    speed_unit: SpeedUnit,
    grade_unit: GradeUnit,
    energy_model_energy_rate_unit: &EnergyRateUnit,
) -> Result<EnergyRate, TraversalModelError> {
    // sweep a fixed set of speed and grade values to find the minimum energy per mile rate from the incoming rf model
    let mut minimum_energy_rate = EnergyRate::from(f64::MAX);
    let start_time = std::time::Instant::now();

    let grade_values = get_grade_sample_values(&grade_unit)?;
    let speed_values = get_speed_sample_values(&speed_unit)?;

    for grade in grade_values.iter() {
        for speed in speed_values.iter() {
            let (energy_rate, _) = model
            .predict(
                (*speed, &speed_unit),
                (*grade, &grade_unit),
            )
            .map_err(|e| TraversalModelError::BuildError(format!("failure while executing grid search for minimum energy rate in prediction model: {}", e)))?;
            if energy_rate < minimum_energy_rate {
                minimum_energy_rate = energy_rate;
            }
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
