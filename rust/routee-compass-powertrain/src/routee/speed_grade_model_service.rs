use super::prediction_model::SpeedGradePredictionModelRecord;
use routee_compass_core::model::traversal::default::speed_lookup_model::get_max_speed;
use routee_compass_core::model::traversal::traversal_model_error::TraversalModelError;
use routee_compass_core::util::fs::read_decoders;
use routee_compass_core::util::fs::read_utils;
use routee_compass_core::util::unit::*;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

#[derive(Clone)]
pub struct SpeedGradeModelService {
    pub speed_table: Arc<Box<[Speed]>>,
    pub speeds_table_speed_unit: SpeedUnit,
    pub max_speed: Speed,
    pub grade_table: Arc<Option<Box<[Grade]>>>,
    pub grade_table_grade_unit: GradeUnit,
    pub output_time_unit: TimeUnit,
    pub output_distance_unit: DistanceUnit,
    pub energy_model_library: HashMap<String, Arc<SpeedGradePredictionModelRecord>>,
}

impl SpeedGradeModelService {
    pub fn new<P: AsRef<Path>>(
        speed_table_path: &P,
        speeds_table_speed_unit: SpeedUnit,
        grade_table_path_option: &Option<P>,
        grade_table_grade_unit_option: Option<GradeUnit>,
        output_time_unit_option: Option<TimeUnit>,
        output_distance_unit_option: Option<DistanceUnit>,
        energy_model_library: HashMap<String, Arc<SpeedGradePredictionModelRecord>>,
    ) -> Result<Self, TraversalModelError> {
        let output_time_unit = output_time_unit_option.unwrap_or(BASE_TIME_UNIT);
        let output_distance_unit = output_distance_unit_option.unwrap_or(BASE_DISTANCE_UNIT);

        // load speeds table
        let speed_table: Arc<Box<[Speed]>> = Arc::new(
            read_utils::read_raw_file(&speed_table_path, read_decoders::default, None).map_err(
                |e| {
                    TraversalModelError::FileReadError(
                        speed_table_path.as_ref().to_path_buf(),
                        e.to_string(),
                    )
                },
            )?,
        );

        let grade_table: Arc<Option<Box<[Grade]>>> = match grade_table_path_option {
            Some(gtp) => Arc::new(Some(
                read_utils::read_raw_file(&gtp, read_decoders::default, None).map_err(|e| {
                    TraversalModelError::FileReadError(
                        speed_table_path.as_ref().to_path_buf(),
                        e.to_string(),
                    )
                })?,
            )),
            None => Arc::new(None),
        };
        let grade_table_grade_unit = grade_table_grade_unit_option.unwrap_or(GradeUnit::Decimal);

        let max_speed = get_max_speed(&speed_table)?;

        Ok(SpeedGradeModelService {
            speed_table,
            speeds_table_speed_unit,
            max_speed,
            grade_table,
            grade_table_grade_unit,
            output_time_unit,
            output_distance_unit,
            energy_model_library,
        })
    }
}
