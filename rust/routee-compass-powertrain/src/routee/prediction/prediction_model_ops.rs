use std::{path::Path, sync::Arc};

use routee_compass_core::{
    model::traversal::traversal_model_error::TraversalModelError,
    util::unit::{EnergyRate, EnergyRateUnit, Grade, GradeUnit, Speed, SpeedUnit},
};

use super::{
    model_type::ModelType, smartcore::smartcore_speed_grade_model::SmartcoreSpeedGradeModel,
    PredictionModel, PredictionModelRecord,
};

#[cfg(feature = "onnx")]
use crate::routee::prediction::onnx::onnx_speed_grade_model::OnnxSpeedGradeModel;

#[allow(clippy::too_many_arguments)]
pub fn load_prediction_model<P: AsRef<Path>>(
    model_name: String,
    model_path: &P,
    model_type: ModelType,
    speed_unit: SpeedUnit,
    grade_unit: GradeUnit,
    energy_rate_unit: EnergyRateUnit,
    ideal_energy_rate_option: Option<EnergyRate>,
    real_world_energy_adjustment_option: Option<f64>,
) -> Result<PredictionModelRecord, TraversalModelError> {
    let prediction_model: Arc<dyn PredictionModel> = match model_type {
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
    };
    let ideal_energy_rate = match ideal_energy_rate_option {
        None => find_min_energy_rate(&prediction_model, &energy_rate_unit)?,
        Some(ier) => ier,
    };

    let real_world_energy_adjustment = real_world_energy_adjustment_option.unwrap_or(1.0);

    Ok(PredictionModelRecord {
        name: model_name,
        prediction_model,
        model_type,
        speed_unit,
        grade_unit,
        energy_rate_unit,
        ideal_energy_rate,
        real_world_energy_adjustment,
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
