use derive_more::{Add, Div, Mul, Neg, Sub, Sum};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, fmt::Display};

use super::{Distance, DistanceUnit, EnergyRate, EnergyRateUnit, EnergyUnit, UnitError};

#[derive(
    Copy,
    Clone,
    Serialize,
    Deserialize,
    PartialEq,
    PartialOrd,
    Eq,
    Hash,
    Debug,
    Default,
    Add,
    Sub,
    Mul,
    Div,
    Sum,
    Neg,
)]
pub struct Energy(pub OrderedFloat<f64>);

impl Ord for Energy {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl Display for Energy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Energy {
    /// calculates an energy value based on some energy rate and distance.
    /// the resulting energy unit is based on the energy rate unit provided.
    pub fn calculate_energy(
        energy_rate: EnergyRate,
        energy_rate_unit: EnergyRateUnit,
        distance: Distance,
        distance_unit: DistanceUnit,
    ) -> Result<(Energy, EnergyUnit), UnitError> {
        let rate_distance_unit = energy_rate_unit.associated_distance_unit();
        let energy_unit = energy_rate_unit.associated_energy_unit();
        let calc_distance = distance_unit.convert(distance, rate_distance_unit);
        let energy_value = energy_rate.to_f64() * calc_distance.to_f64();
        let energy = Energy::new(energy_value);
        Ok((energy, energy_unit))
    }

    pub fn new(value: f64) -> Energy {
        Energy(OrderedFloat(value))
    }
    pub fn to_f64(&self) -> f64 {
        (self.0).0
    }
    pub const ZERO: Energy = Energy(OrderedFloat(0.0));
    pub const ONE: Energy = Energy(OrderedFloat(1.0));
}
