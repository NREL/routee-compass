use routee_compass_core::model::unit::{Distance, DistanceUnit, Weight, WeightUnit};
use serde::{Deserialize, Serialize};

use super::truck_parameters;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TruckRestriction {
    MaximumTotalWeight((Weight, WeightUnit)),
    MaximumWeightPerAxle((Weight, WeightUnit)),
    MaximumLength((Distance, DistanceUnit)),
    MaximumWidth((Distance, DistanceUnit)),
    MaximumHeight((Distance, DistanceUnit)),
    MaximumTrailerLength((Distance, DistanceUnit)),
}

impl TruckRestriction {
    pub fn validate(&self, truck_parameters: &truck_parameters::TruckParameters) -> bool {
        match self {
            TruckRestriction::MaximumTotalWeight((restriction_weight, restriction_unit)) => {
                let (truck_weight, truck_unit) = truck_parameters.truck_total_weight;
                let weight_in_restriction_unit =
                    truck_unit.convert(&truck_weight, restriction_unit);
                weight_in_restriction_unit <= *restriction_weight
            }
            TruckRestriction::MaximumWeightPerAxle((restriction_weight, restriction_unit)) => {
                let (truck_weight, truck_unit) = truck_parameters.truck_total_weight;
                let weight_in_restriction_unit =
                    truck_unit.convert(&truck_weight, restriction_unit);
                let weight_per_axle =
                    weight_in_restriction_unit / truck_parameters.truck_number_of_axles as f64;
                weight_per_axle <= *restriction_weight
            }
            TruckRestriction::MaximumLength((restriction_length, restriction_unit)) => {
                let (truck_length, truck_unit) = truck_parameters.truck_total_length;
                let length_in_restriction_unit =
                    truck_unit.convert(&truck_length, restriction_unit);
                length_in_restriction_unit <= *restriction_length
            }
            TruckRestriction::MaximumWidth((restriction_width, restriction_unit)) => {
                let (truck_width, truck_unit) = truck_parameters.truck_width;
                let width_in_restriction_unit = truck_unit.convert(&truck_width, restriction_unit);
                width_in_restriction_unit <= *restriction_width
            }
            TruckRestriction::MaximumHeight((restriction_height, restriction_unit)) => {
                let (truck_height, truck_unit) = truck_parameters.truck_height;
                let height_in_restriction_unit =
                    truck_unit.convert(&truck_height, restriction_unit);
                height_in_restriction_unit <= *restriction_height
            }
            TruckRestriction::MaximumTrailerLength((restriction_length, restriction_unit)) => {
                let (truck_trailer_length, truck_unit) = truck_parameters.truck_trailer_length;
                let trailer_length_in_restriction_unit =
                    truck_unit.convert(&truck_trailer_length, restriction_unit);
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
        let restriction: TruckRestriction = serde_json::from_str(json).unwrap();
        assert_eq!(
            restriction,
            TruckRestriction::MaximumTotalWeight((Weight::new(1000.0), WeightUnit::Kg))
        );
    }
}
