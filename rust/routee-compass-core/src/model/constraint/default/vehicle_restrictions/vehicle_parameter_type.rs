use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum VehicleParameterType {
    Height,
    Width,
    TotalLength,
    TrailerLength,
    TotalWeight,
    WeightPerAxle,
}

impl std::fmt::Display for VehicleParameterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Height => "height".to_string(),
            Self::Width => "width".to_string(),
            Self::TotalLength => "total_length".to_string(),
            Self::TrailerLength => "trailer_length".to_string(),
            Self::TotalWeight => "total_weight".to_string(),
            Self::WeightPerAxle => "weight_per_axle".to_string(),
        };
        write!(f, "{s}")
    }
}

impl FromStr for VehicleParameterType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "height" => Ok(Self::Height),
            "width" => Ok(Self::Width),
            "total_length" => Ok(Self::TotalLength),
            "trailer_length" => Ok(Self::TrailerLength),
            "total_weight" => Ok(Self::TotalWeight),
            "weight_per_axle" => Ok(Self::WeightPerAxle),
            _ => Err(format!("unknown VehicleParameterType {s}")),
        }
    }
}
