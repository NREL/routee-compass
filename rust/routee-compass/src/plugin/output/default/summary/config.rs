use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SummaryConfig {
    pub estimate_memory_consumption: Option<bool>,
}
