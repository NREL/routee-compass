use allocative::Allocative;
use derive_more::{Add, Div, Mul, Neg, Sub, Sum};
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, fmt::Display};

use super::{
    internal_float::InternalFloat, AsF64, Distance, DistanceUnit, Energy, EnergyRateUnit,
    EnergyUnit,
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
    Allocative,
)]
pub struct EnergyRate(pub InternalFloat);

impl EnergyRate {
    pub fn from_energy_and_distance(
        e: (&Energy, &EnergyUnit),
        d: (&Distance, &DistanceUnit),
        energy_per_distance_format: bool,
    ) -> (EnergyRate, EnergyRateUnit) {
        let (energy, eu) = e;
        let (distance, du) = d;
        if energy_per_distance_format {
            let er = EnergyRate::from(energy.as_f64() / distance.as_f64());
            let eru = EnergyRateUnit::EnergyPerDistance(*eu, *du);
            (er, eru)
        } else {
            let er = EnergyRate::from(distance.as_f64() / energy.as_f64());
            let eru = EnergyRateUnit::DistancePerEnergy(*du, *eu);
            (er, eru)
        }
    }
}

impl From<f64> for EnergyRate {
    fn from(value: f64) -> Self {
        EnergyRate(InternalFloat::new(value))
    }
}

impl AsF64 for EnergyRate {
    fn as_f64(&self) -> f64 {
        (self.0).0
    }
}

impl AsF64 for &EnergyRate {
    fn as_f64(&self) -> f64 {
        (self.0).0
    }
}

impl PartialOrd for EnergyRate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.0.cmp(&other.0))
    }
}

impl Ord for EnergyRate {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl Display for EnergyRate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl EnergyRate {
    pub const ZERO: EnergyRate = EnergyRate(InternalFloat::ZERO);
    pub const ONE: EnergyRate = EnergyRate(InternalFloat::ONE);
}
