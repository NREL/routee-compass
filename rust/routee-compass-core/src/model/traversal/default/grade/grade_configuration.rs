use crate::model::unit::GradeUnit;
use serde::{Deserialize, Serialize};

/// provides configuration for instantiating the grade engine used in grade modeling.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GradeConfiguration {
    /// file with dense mapping from edge id to grade value
    /// note: this is optional as a stop-gap to allow for loading energy models that do not require grade.
    /// this is because we currently cannot define input features that are _optional_, and so
    /// if a dummy grade model were not provided, energy models would fail to build.
    pub grade_input_file: Option<String>,
    /// type of grade values in file
    pub grade_unit: GradeUnit,
}
