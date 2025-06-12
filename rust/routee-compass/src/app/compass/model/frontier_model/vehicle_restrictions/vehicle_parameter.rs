use super::VehicleParameterConfig;
use serde::Deserialize;
use uom::si::f64::{Length, Mass};

#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize)]
pub enum VehicleParameter {
    Height { value: Length },
    Width { value: Length },
    TotalLength { value: Length },
    TrailerLength { value: Length },
    TotalWeight { value: Mass },
    WeightPerAxle { value: Mass },
}

impl VehicleParameter {
    pub fn name(&self) -> String {
        match self {
            VehicleParameter::Height { .. } => "height".to_string(),
            VehicleParameter::Width { .. } => "width".to_string(),
            VehicleParameter::TotalLength { .. } => "total_length".to_string(),
            VehicleParameter::TrailerLength { .. } => "trailer_length".to_string(),
            VehicleParameter::TotalWeight { .. } => "total_weight".to_string(),
            VehicleParameter::WeightPerAxle { .. } => "weight_per_axle".to_string(),
        }
    }
}


impl std::fmt::Display for VehicleParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VehicleParameter::Height { value } => write!(f, "height: {:?}", value),
            VehicleParameter::Width { value } => write!(f, "width: {:?}", value),
            VehicleParameter::TotalLength { value } => {
                write!(f, "total length: {:?}", value)
            }
            VehicleParameter::TrailerLength { value } => {
                write!(f, "trailer length: {:?}", value)
            }
            VehicleParameter::TotalWeight { value } => {
                write!(f, "total weight: {:?}", value)
            }
            VehicleParameter::WeightPerAxle { value } => {
                write!(f, "weight per axle: {:?}", value)
            }
        }
    }
}
