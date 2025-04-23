use crate::model::unit::{DistanceUnit, GradeUnit};
use serde::{Deserialize, Serialize};

/// provides configuration for instantiating the grade engine used in grade modeling.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GradeConfiguration {
    /// file with dense mapping from edge id to grade value
    pub grade_input_file: String,
    /// type of grade values in file
    pub grade_unit: GradeUnit,
    /// distance unit used when recording elevation values. if not provided,
    /// application will use "feet".
    pub elevation_unit: Option<DistanceUnit>,
}
