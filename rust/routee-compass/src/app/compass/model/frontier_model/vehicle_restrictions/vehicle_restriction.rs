use std::borrow::Cow;

use routee_compass_core::model::unit::{
    Convert, Distance, DistanceUnit, UnitError, Weight, WeightUnit,
};
use serde::{Deserialize, Serialize};

use super::vehicle_parameters;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VehicleRestriction {
    MaximumTotalWeight((Weight, WeightUnit)),
    MaximumWeightPerAxle((Weight, WeightUnit)),
    MaximumLength((Distance, DistanceUnit)),
    MaximumWidth((Distance, DistanceUnit)),
    MaximumHeight((Distance, DistanceUnit)),
    MaximumTrailerLength((Distance, DistanceUnit)),
}

impl VehicleRestriction {
    /// Returns true if the truck parameters are valid for the restriction.
    /// For example, if the restriction is MaximumTotalWeight(1000.0, "kg"),
    /// and the truck parameters are VehicleParameters { vehicle_total_weight: (500.0, "kg"), ... },
    /// then the function will return true.
    pub fn valid(
        &self,
        vehicle_parameters: &vehicle_parameters::VehicleParameters,
    ) -> Result<bool, UnitError> {
        let result = match self {
            VehicleRestriction::MaximumTotalWeight((restriction_weight, restriction_unit)) => {
                let (vehicle_weight, weight_unit) = vehicle_parameters.total_weight;

                let mut weight_in_restriction_unit = Cow::Owned(vehicle_weight);
                weight_unit.convert(&mut weight_in_restriction_unit, restriction_unit)?;
                weight_in_restriction_unit.into_owned() <= *restriction_weight
            }
            VehicleRestriction::MaximumWeightPerAxle((restriction_weight, restriction_unit)) => {
                let (vehicle_weight, weight_unit) = vehicle_parameters.total_weight;
                let mut weight_in_restriction_unit = Cow::Owned(vehicle_weight);
                weight_unit.convert(&mut weight_in_restriction_unit, restriction_unit)?;
                let weight_per_axle = weight_in_restriction_unit.into_owned()
                    / vehicle_parameters.number_of_axles as f64;
                weight_per_axle <= *restriction_weight
            }
            VehicleRestriction::MaximumLength((restriction_length, restriction_unit)) => {
                let (vehicle_length, vehicle_unit) = vehicle_parameters.total_length;
                let mut length_in_restriction_unit = Cow::Owned(vehicle_length);
                vehicle_unit.convert(&mut length_in_restriction_unit, restriction_unit)?;
                length_in_restriction_unit.into_owned() <= *restriction_length
            }
            VehicleRestriction::MaximumWidth((restriction_width, restriction_unit)) => {
                let (vehicle_width, vehicle_unit) = vehicle_parameters.width;
                let mut width_in_restriction_unit = Cow::Owned(vehicle_width);
                vehicle_unit.convert(&mut width_in_restriction_unit, restriction_unit)?;
                width_in_restriction_unit.into_owned() <= *restriction_width
            }
            VehicleRestriction::MaximumHeight((restriction_height, restriction_unit)) => {
                let (vehicle_height, vehicle_unit) = vehicle_parameters.height;
                let mut height_in_restriction_unit = Cow::Owned(vehicle_height);
                vehicle_unit.convert(&mut height_in_restriction_unit, restriction_unit)?;
                height_in_restriction_unit.into_owned() <= *restriction_height
            }
            VehicleRestriction::MaximumTrailerLength((restriction_length, restriction_unit)) => {
                let (vehicle_trailer_length, vehicle_unit) = vehicle_parameters.trailer_length;
                let mut trailer_length_in_restriction_unit = Cow::Owned(vehicle_trailer_length);
                vehicle_unit.convert(&mut trailer_length_in_restriction_unit, restriction_unit)?;
                trailer_length_in_restriction_unit.into_owned() <= *restriction_length
            }
        };
        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_deserialize() {
        let json = r#"{"maximum_total_weight":[1000.0,"kg"]}"#;
        let restriction: VehicleRestriction = serde_json::from_str(json).unwrap();
        assert_eq!(
            restriction,
            VehicleRestriction::MaximumTotalWeight((Weight::from(1000.0), WeightUnit::Kg))
        );
    }
}
