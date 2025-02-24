use super::{
    internal_float::InternalFloat, AsF64, Convert, Distance, DistanceUnit, EnergyRate,
    EnergyRateUnit, EnergyUnit, UnitError,
};
use crate::model::state::StateVariable;
use allocative::Allocative;
use derive_more::{Add, Div, Mul, Neg, Sub, Sum};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, cmp::Ordering, fmt::Display};

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
pub struct Energy(pub InternalFloat);

impl From<StateVariable> for Energy {
    fn from(value: StateVariable) -> Self {
        Energy::from(value.0)
    }
}

impl From<&StateVariable> for Energy {
    fn from(value: &StateVariable) -> Self {
        Energy::from(value.0)
    }
}

impl From<f64> for Energy {
    fn from(value: f64) -> Energy {
        Energy(InternalFloat::new(value))
    }
}

impl AsF64 for Energy {
    fn as_f64(&self) -> f64 {
        (self.0).0
    }
}

impl AsF64 for &Energy {
    fn as_f64(&self) -> f64 {
        (self.0).0
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
        write!(f, "{:?}", self.0)
    }
}

impl Energy {
    /// calculates an energy value based on some energy rate and distance.
    /// the resulting energy unit is based on the energy rate unit provided.
    pub fn create(
        distance: (&Distance, &DistanceUnit),
        energy_rate: (&EnergyRate, &EnergyRateUnit),
    ) -> Result<(Energy, EnergyUnit), UnitError> {
        let (er, eru) = energy_rate;
        let (d, du) = distance;
        let associated_distance_unit = eru.associated_distance_unit();
        let associated_energy_unit = eru.associated_energy_unit();

        let mut d_cow = Cow::Borrowed(d);
        du.convert(&mut d_cow, &associated_distance_unit)?;

        let energy = Energy::from(er.as_f64() * d_cow.as_ref().as_f64());
        Ok((energy, associated_energy_unit))
    }
    pub const ZERO: Energy = Energy(InternalFloat::ZERO);
    pub const ONE: Energy = Energy(InternalFloat::ONE);
}

#[cfg(test)]
mod tests {
    use crate::model::unit::*;

    fn approx_eq_energy(a: Energy, b: Energy, error: f64) {
        let result = match (a, b) {
            (c, d) if c < d => (d - c).as_f64() < error,
            (c, d) if c > d => (c - d).as_f64() < error,
            (_, _) => true,
        };
        assert!(
            result,
            "{} ~= {} is not true within an error of {}",
            a, b, error
        )
    }

    #[test]
    fn test_energy_ggpm_meters() {
        let ten_mpg_rate = 1.0 / 10.0;
        let (energy, energy_unit) = Energy::create(
            (&Distance::from(1609.0), &DistanceUnit::Meters),
            (
                &EnergyRate::from(ten_mpg_rate),
                &EnergyRateUnit::EnergyPerDistance(
                    EnergyUnit::GallonsGasoline,
                    DistanceUnit::Miles,
                ),
            ),
        )
        .unwrap();
        approx_eq_energy(energy, Energy::from(ten_mpg_rate), 0.00001);
        assert_eq!(energy_unit, EnergyUnit::GallonsGasoline);
    }

    #[test]
    fn test_energy_ggpm_miles() {
        let ten_mpg_rate = 1.0 / 10.0;
        let (energy, energy_unit) = Energy::create(
            (&Distance::from(1.0), &DistanceUnit::Miles),
            (
                &EnergyRate::from(ten_mpg_rate),
                &EnergyRateUnit::EnergyPerDistance(
                    EnergyUnit::GallonsGasoline,
                    DistanceUnit::Miles,
                ),
            ),
        )
        .unwrap();
        approx_eq_energy(energy, Energy::from(ten_mpg_rate), 0.00001);
        assert_eq!(energy_unit, EnergyUnit::GallonsGasoline);
    }
}
