use std::{path::Path, sync::Arc};

use routee_compass_core::{
    model::traversal::traversal_model_error::TraversalModelError,
    model::unit::{EnergyRate, EnergyRateUnit, Grade, GradeUnit, Speed, SpeedUnit},
    util::cache_policy::float_cache_policy::FloatCachePolicy,
};

use super::{
    interpolation::interpolation_speed_grade_model::InterpolationSpeedGradeModel,
    model_type::ModelType, smartcore::smartcore_speed_grade_model::SmartcoreSpeedGradeModel,
    PredictionModel, PredictionModelRecord,
};

#[cfg(feature = "onnx")]
use crate::routee::prediction::onnx::onnx_speed_grade_model::OnnxSpeedGradeModel;

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
        None => find_min_energy_rate(&prediction_model, &energy_rate_unit)?,
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

/// sweep a fixed set of speed and grade values to find the minimum energy per mile rate from the incoming rf model
pub fn find_min_energy_rate(
    model: &Arc<dyn PredictionModel>,
    energy_model_energy_rate_unit: &EnergyRateUnit,
) -> Result<EnergyRate, TraversalModelError> {
    // sweep a fixed set of speed and grade values to find the minimum energy per mile rate from the incoming rf model
    let mut minimum_energy_rate = EnergyRate::new(f64::MAX);
    let start_time = std::time::Instant::now();

    let grade = Grade::ZERO;
    for speed_i32 in 20..80 {
        let speed = Speed::new(speed_i32 as f64);
        let (energy_rate, _) = model
            .predict(
                (speed, SpeedUnit::MilesPerHour),
                (grade, GradeUnit::Percent),
            )
            .map_err(|e| TraversalModelError::PredictionModel(e.to_string()))?;
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
