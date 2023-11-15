use derive_more::{Add, Div, Mul, Neg, Sub, Sum};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, fmt::Display};

use super::{
    as_f64::AsF64, builders::create_energy, Distance, DistanceUnit, EnergyRate, EnergyRateUnit,
    EnergyUnit, UnitError,
};

#[derive(
    Copy,
    Clone,
    Serialize,
    Deserialize,
    PartialEq,
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

impl AsF64 for Energy {
    fn as_f64(&self) -> f64 {
        (self.0).0
    }
}

impl From<(EnergyRate, Distance)> for Energy {
    fn from(value: (EnergyRate, Distance)) -> Self {
        let (energy_rate, distance) = value;
        let energy_value = energy_rate.as_f64() * distance.as_f64();
        Energy::new(energy_value)
    }
}

impl PartialOrd for Energy {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.0.cmp(&other.0))
    }
}

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
    pub fn new(value: f64) -> Energy {
        Energy(OrderedFloat(value))
    }
    pub fn create(
        energy_rate: EnergyRate,
        energy_rate_unit: EnergyRateUnit,
        distance: Distance,
        distance_unit: DistanceUnit,
    ) -> Result<(Energy, EnergyUnit), UnitError> {
        create_energy(energy_rate, energy_rate_unit, distance, distance_unit)
    }
    pub const ZERO: Energy = Energy(OrderedFloat(0.0));
    pub const ONE: Energy = Energy(OrderedFloat(1.0));
}
