use serde::Deserialize;
use uom::si::f64::{Length, Mass};

use crate::model::constraint::default::vehicle_restrictions::VehicleParameterType;

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
    pub fn vehicle_parameter_type(&self) -> &VehicleParameterType {
        use VehicleParameterType as VPT;
        match self {
            VehicleParameter::Height { .. } => &VPT::Height,
            VehicleParameter::Width { .. } => &VPT::Width,
            VehicleParameter::TotalLength { .. } => &VPT::TotalLength,
            VehicleParameter::TrailerLength { .. } => &VPT::TrailerLength,
            VehicleParameter::TotalWeight { .. } => &VPT::TotalWeight,
            VehicleParameter::WeightPerAxle { .. } => &VPT::WeightPerAxle,
        }
    }
}

impl std::fmt::Display for VehicleParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VehicleParameter::Height { value } => write!(f, "height: {value:?}"),
            VehicleParameter::Width { value } => write!(f, "width: {value:?}"),
            VehicleParameter::TotalLength { value } => {
                write!(f, "total length: {value:?}")
            }
            VehicleParameter::TrailerLength { value } => {
                write!(f, "trailer length: {value:?}")
            }
            VehicleParameter::TotalWeight { value } => {
                write!(f, "total weight: {value:?}")
            }
            VehicleParameter::WeightPerAxle { value } => {
                write!(f, "weight per axle: {value:?}")
            }
        }
    }
}
