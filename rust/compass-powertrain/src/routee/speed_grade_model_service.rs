use super::model_type::ModelType;
use super::prediction_model::SpeedGradePredictionModel;
use compass_core::model::graph::edge_id::EdgeId;
use compass_core::model::traversal::default::speed_lookup_model::get_max_speed;
use compass_core::model::traversal::traversal_model_error::TraversalModelError;
use compass_core::util::fs::read_decoders;
use compass_core::util::fs::read_utils;
use compass_core::util::unit::*;
use std::sync::Arc;

#[derive(Clone)]
pub struct SpeedGradeModelService {
    pub speed_table: Arc<Vec<Speed>>,
    pub speeds_table_speed_unit: SpeedUnit,
    pub max_speed: Speed,
    pub energy_model: Arc<dyn SpeedGradePredictionModel>,
    pub energy_model_energy_rate_unit: EnergyRateUnit,
    pub energy_model_speed_unit: SpeedUnit,
    pub output_time_unit: TimeUnit,
    pub minimum_energy_rate: EnergyRate,
}

impl SpeedGradeModelService {
    pub fn new(
        speed_table_path: String,
        speeds_table_speed_unit: SpeedUnit,
        energy_model_path: String,
        model_type: ModelType,
        energy_model_speed_unit: SpeedUnit,
        energy_model_energy_rate_unit: EnergyRateUnit,
        output_time_unit_option: Option<TimeUnit>,
    ) -> Result<Self, TraversalModelError> {
        let output_time_unit = output_time_unit_option.unwrap_or(BASE_TIME_UNIT);

        let energy_model = model_type.build(
            energy_model_path.clone(),
            energy_model_speed_unit.clone(),
            energy_model_energy_rate_unit.clone(),
        )?;

        let minimum_energy_rate = find_min_energy_rate(
            &energy_model,
            &energy_model_speed_unit,
            &energy_model_energy_rate_unit,
        )?;

        // load speeds table
        let speed_table: Arc<Vec<Speed>> = Arc::new(
            read_utils::read_raw_file(&speed_table_path, read_decoders::default, None).map_err(
                |e| TraversalModelError::FileReadError(speed_table_path.clone(), e.to_string()),
            )?,
        );
        let max_speed = get_max_speed(&speed_table)?;

        Ok(SpeedGradeModelService {
            speed_table,
            speeds_table_speed_unit,
            max_speed,
            energy_model,
            energy_model_energy_rate_unit,
            energy_model_speed_unit,
            output_time_unit,
            minimum_energy_rate,
        })
    }
}

/// sweep a fixed set of speed and grade values to find the minimum energy per mile rate from the incoming rf model
pub fn find_min_energy_rate(
    model: &Arc<dyn SpeedGradePredictionModel>,
    energy_model_speed_unit: &SpeedUnit,
    energy_model_energy_rate_unit: &EnergyRateUnit,
) -> Result<EnergyRate, TraversalModelError> {
    // sweep a fixed set of speed and grade values to find the minimum energy per mile rate from the incoming rf model
    let max_speed = energy_model_speed_unit.max_american_highway_speed();
    let max_speed_i32 = max_speed.to_f64().ceil() as i32;
    let mut minimum_energy_rate = EnergyRate::new(f64::MAX);
    let start_time = std::time::Instant::now();

    for speed_i32 in 1..max_speed_i32 {
        for grade_percent in -20..20 {
            let speed = Speed::new(speed_i32 as f64);
            let grade = grade_percent as f64;
            let (energy_rate, _) = model
                .predict(speed, energy_model_speed_unit.clone(), grade)
                .map_err(|e| TraversalModelError::PredictionModel(e.to_string()))?;
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
