use routee_compass_core::model::unit::{Distance, DistanceUnit, Weight, WeightUnit};
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
    pub fn valid(&self, vehicle_parameters: &vehicle_parameters::VehicleParameters) -> bool {
        match self {
            VehicleRestriction::MaximumTotalWeight((restriction_weight, restriction_unit)) => {
                let (vehicle_weight, vehicle_unit) = vehicle_parameters.total_weight;
                let weight_in_restriction_unit =
                    vehicle_unit.convert(&vehicle_weight, restriction_unit);
                weight_in_restriction_unit <= *restriction_weight
            }
            VehicleRestriction::MaximumWeightPerAxle((restriction_weight, restriction_unit)) => {
                let (vehicle_weight, vehicle_unit) = vehicle_parameters.total_weight;
                let weight_in_restriction_unit =
                    vehicle_unit.convert(&vehicle_weight, restriction_unit);
                let weight_per_axle =
                    weight_in_restriction_unit / vehicle_parameters.number_of_axles as f64;
                weight_per_axle <= *restriction_weight
            }
            VehicleRestriction::MaximumLength((restriction_length, restriction_unit)) => {
                let (vehicle_length, vehicle_unit) = vehicle_parameters.total_length;
                let length_in_restriction_unit =
                    vehicle_unit.convert(&vehicle_length, restriction_unit);
                length_in_restriction_unit <= *restriction_length
            }
            VehicleRestriction::MaximumWidth((restriction_width, restriction_unit)) => {
                let (vehicle_width, vehicle_unit) = vehicle_parameters.width;
                let width_in_restriction_unit =
                    vehicle_unit.convert(&vehicle_width, restriction_unit);
                width_in_restriction_unit <= *restriction_width
            }
            VehicleRestriction::MaximumHeight((restriction_height, restriction_unit)) => {
                let (vehicle_height, vehicle_unit) = vehicle_parameters.height;
                let height_in_restriction_unit =
                    vehicle_unit.convert(&vehicle_height, restriction_unit);
                height_in_restriction_unit <= *restriction_height
            }
            VehicleRestriction::MaximumTrailerLength((restriction_length, restriction_unit)) => {
                let (vehicle_trailer_length, vehicle_unit) = vehicle_parameters.trailer_length;
                let trailer_length_in_restriction_unit =
                    vehicle_unit.convert(&vehicle_trailer_length, restriction_unit);
                trailer_length_in_restriction_unit <= *restriction_length
            }
        }
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
            VehicleRestriction::MaximumTotalWeight((Weight::new(1000.0), WeightUnit::Kg))
        );
    }
}
