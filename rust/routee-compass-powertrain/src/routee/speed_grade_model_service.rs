use crate::routee::smartcore::smartcore_speed_grade_model::SmartcoreSpeedGradeModel;

use super::model_type::ModelType;
use super::prediction_model::SpeedGradePredictionModel;
use routee_compass_core::model::traversal::default::speed_lookup_model::get_max_speed;
use routee_compass_core::model::traversal::traversal_model_error::TraversalModelError;
use routee_compass_core::util::fs::read_decoders;
use routee_compass_core::util::fs::read_utils;
use routee_compass_core::util::unit::*;
use std::sync::Arc;

#[cfg(feature = "onnx")]
use crate::routee::onnx::onnx_speed_grade_model::OnnxSpeedGradeModel;

#[derive(Clone)]
pub struct SpeedGradeModelService {
    pub speed_table: Arc<Vec<Speed>>,
    pub speeds_table_speed_unit: SpeedUnit,
    pub max_speed: Speed,
    pub grade_table: Arc<Option<Vec<Grade>>>,
    pub grade_table_grade_unit: GradeUnit,
    pub energy_model: Arc<dyn SpeedGradePredictionModel>,
    pub energy_model_energy_rate_unit: EnergyRateUnit,
    pub energy_model_speed_unit: SpeedUnit,
    pub energy_model_grade_unit: GradeUnit,
    pub output_time_unit: TimeUnit,
    pub output_distance_unit: DistanceUnit,
    pub ideal_energy_rate: EnergyRate,
    pub real_world_energy_adjustment: f64,
}

impl SpeedGradeModelService {
    pub fn new(
        speed_table_path: String,
        speeds_table_speed_unit: SpeedUnit,
        grade_table_path_option: Option<String>,
        grade_table_grade_unit_option: Option<GradeUnit>,
        energy_model_path: String,
        model_type: ModelType,
        ideal_energy_rate_option: Option<EnergyRate>,
        energy_model_speed_unit: SpeedUnit,
        energy_model_grade_unit: GradeUnit,
        energy_model_energy_rate_unit: EnergyRateUnit,
        output_time_unit_option: Option<TimeUnit>,
        output_distance_unit_option: Option<DistanceUnit>,
        real_world_energy_adjustment: Option<f64>,
    ) -> Result<Self, TraversalModelError> {
        let output_time_unit = output_time_unit_option.unwrap_or(BASE_TIME_UNIT);
        let output_distance_unit = output_distance_unit_option.unwrap_or(BASE_DISTANCE_UNIT);

        let energy_model = model_type.build(
            energy_model_path.clone(),
            energy_model_speed_unit.clone(),
            energy_model_grade_unit.clone(),
            energy_model_energy_rate_unit.clone(),
        )?;

        let ideal_energy_rate = match ideal_energy_rate_option {
            None => find_min_energy_rate(&energy_model, &energy_model_energy_rate_unit)?,
            Some(ier) => ier,
        };

        // load speeds table
        let speed_table: Arc<Vec<Speed>> = Arc::new(
            read_utils::read_raw_file(&speed_table_path, read_decoders::default, None).map_err(
                |e| TraversalModelError::FileReadError(speed_table_path.clone(), e.to_string()),
            )?,
        );

        let grade_table: Arc<Option<Vec<Grade>>> = match grade_table_path_option {
            Some(gtp) => Arc::new(Some(
                read_utils::read_raw_file(&gtp, read_decoders::default, None).map_err(|e| {
                    TraversalModelError::FileReadError(speed_table_path.clone(), e.to_string())
                })?,
            )),
            None => Arc::new(None),
        };
        let grade_table_grade_unit = grade_table_grade_unit_option.unwrap_or(GradeUnit::Decimal);

        // Load random forest binary file
        let energy_model: Arc<dyn SpeedGradePredictionModel> = match model_type {
            ModelType::Smartcore => {
                let model = SmartcoreSpeedGradeModel::new(
                    energy_model_path.clone(),
                    energy_model_speed_unit,
                    energy_model_grade_unit,
                    energy_model_energy_rate_unit,
                )?;
                Arc::new(model)
            }
            ModelType::Onnx => {
                #[cfg(feature = "onnx")]
                {
                    let model = OnnxSpeedGradeModel::new(
                        energy_model_path.clone(),
                        energy_model_speed_unit,
                        energy_model_grade_unit,
                        energy_model_energy_rate_unit,
                    )?;
                    Arc::new(model)
                }
                #[cfg(not(feature = "onnx"))]
                {
                    return Err(
                    TraversalModelError::BuildError(
                        "Cannot build Onnx model without `onnx` feature enabled for compass-powertrain"
                            .to_string(),
                    )
                );
                }
            }
        };

        let max_speed = get_max_speed(&speed_table)?;

        let real_world_energy_adjustment = match real_world_energy_adjustment {
            Some(rwea) => rwea,
            None => 1.0,
        };

        Ok(SpeedGradeModelService {
            speed_table,
            speeds_table_speed_unit,
            max_speed,
            grade_table,
            grade_table_grade_unit,
            energy_model,
            energy_model_energy_rate_unit,
            energy_model_speed_unit,
            energy_model_grade_unit,
            output_time_unit,
            output_distance_unit,
            ideal_energy_rate,
            real_world_energy_adjustment,
        })
    }
}

/// sweep a fixed set of speed and grade values to find the minimum energy per mile rate from the incoming rf model
pub fn find_min_energy_rate(
    model: &Arc<dyn SpeedGradePredictionModel>,
    energy_model_energy_rate_unit: &EnergyRateUnit,
) -> Result<EnergyRate, TraversalModelError> {
    // sweep a fixed set of speed and grade values to find the minimum energy per mile rate from the incoming rf model
    let mut minimum_energy_rate = EnergyRate::new(f64::MAX);
    let start_time = std::time::Instant::now();

    let grade = Grade::ZERO;
    for speed_i32 in 20..80 {
        let speed = Speed::new(speed_i32 as f64);
        let (energy_rate, _) = model
            .predict(speed, SpeedUnit::MilesPerHour, grade, GradeUnit::Percent)
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
