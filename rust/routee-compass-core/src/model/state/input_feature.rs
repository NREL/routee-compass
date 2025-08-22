use serde::{Deserialize, Serialize};

use crate::model::unit::{
    DistanceUnit, EnergyUnit, RatioUnit, SpeedUnit, TemperatureUnit, TimeUnit,
};

/// defines the required input feature and its requested unit type for a given state variable
///
/// if a unit type is provided, then the state variable is provided in the requested unit to the model.
#[derive(Serialize, Deserialize, Clone, Debug, Hash, Eq, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InputFeature {
    Distance {
        name: String,
        unit: Option<DistanceUnit>,
    },
    Speed {
        name: String,
        unit: Option<SpeedUnit>,
    },
    Time {
        name: String,
        unit: Option<TimeUnit>,
    },
    Energy {
        name: String,
        unit: Option<EnergyUnit>,
    },
    Ratio {
        name: String,
        unit: Option<RatioUnit>,
    },
    Temperature {
        name: String,
        unit: Option<TemperatureUnit>,
    },
    Custom {
        name: String,
        unit: String,
    },
}

impl InputFeature {
    pub fn name(&self) -> String {
        match self {
            InputFeature::Distance { name, .. } => name.to_owned(),
            InputFeature::Speed { name, .. } => name.to_owned(),
            InputFeature::Time { name, .. } => name.to_owned(),
            InputFeature::Energy { name, .. } => name.to_owned(),
            InputFeature::Ratio { name, .. } => name.to_owned(),
            InputFeature::Temperature { name, .. } => name.to_owned(),
            InputFeature::Custom { name, .. } => name.to_owned(),
        }
    }
}

impl std::fmt::Display for InputFeature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_string_pretty(self).unwrap_or_default();
        write!(f, "{s}")
    }
}
