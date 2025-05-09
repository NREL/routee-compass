use crate::model::unit::SpeedUnit;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SpeedConfiguration {
    /// file containing speed values for each edge id
    speed_table_input_file: String,
    /// unit the speeds were recorded in
    speed_unit: SpeedUnit,
}
