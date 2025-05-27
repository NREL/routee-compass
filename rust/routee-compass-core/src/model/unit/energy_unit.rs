use super::{baseunit, Convert, Energy, UnitError, VolumeUnit};
use crate::model::unit::{AsF64, Volume};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, str::FromStr};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Copy)]
#[serde(rename_all = "snake_case", try_from = "String")]
pub enum EnergyUnit {
    /// electric fuel
    KilowattHours,
    /// 1 [VolumeUnit] Gasoline fuel
    Gasoline(VolumeUnit),
    /// 1 [VolumeUnit] Diesel fuel
    Diesel(VolumeUnit),
    /// unit representing either electric or liquid fuel
    GallonsGasolineEquivalent,
    GallonsDieselEquivalent,
    /// Other commonly-used energy units
    KiloJoules,
    BTU,
}

fn get_volume_factor(from: &VolumeUnit, to: &VolumeUnit) -> Result<f64, UnitError> {
    let mut volume_value = Cow::<Volume>::Owned(Volume::ONE);
    from.convert(&mut volume_value, to)?;
    Ok(volume_value.as_f64())
}

#[allow(non_upper_case_globals)]
impl Convert<Energy> for EnergyUnit {
    fn convert(&self, value: &mut std::borrow::Cow<Energy>, to: &Self) -> Result<(), UnitError> {
        use EnergyUnit as EU;
        use VolumeUnit as V;

        const Gas2Die: f64 = 33.41 / 37.95;
        const Die2Gas: f64 = 37.95 / 33.41;
        const Gas2Kwh: f64 = 33.41;
        const kWh2Gas: f64 = 1. / 33.41;
        const kWh2Kj: f64 = 3_600.;
        const Kj2Kwh: f64 = 1. / 3600.;
        const kWh2BTU: f64 = 3_412.14;
        const BTU2Kwh: f64 = 1. / 3412.14;

        let conversion_factor = match (self, to) {
            // Variants that do not need transformation
            (x, y) if x == y => None,
            (EU::Gasoline(V::GallonsUs), EU::GallonsGasolineEquivalent)
            | (EU::GallonsGasolineEquivalent, EU::Gasoline(V::GallonsUs)) => None,
            (EU::Diesel(V::GallonsUs), EU::GallonsDieselEquivalent)
            | (EU::GallonsDieselEquivalent, EU::Diesel(V::GallonsUs)) => None,

            // Volume to Volume Variants
            (EU::Gasoline(volume_from), EU::Gasoline(volume_to))
            | (EU::Diesel(volume_from), EU::Diesel(volume_to)) => {
                Some(get_volume_factor(volume_from, volume_to)?)
            }
            // Same volume Unit
            (EU::Gasoline(volume_from), EU::Diesel(volume_to)) if volume_from == volume_to => {
                Some(Gas2Die)
            }
            (EU::Diesel(volume_from), EU::Gasoline(volume_to)) if volume_from == volume_to => {
                Some(Die2Gas)
            }
            // Different liquid diffent volume
            (EU::Gasoline(volume_from), EU::Diesel(volume_to)) => {
                Some(get_volume_factor(volume_from, volume_to)? * Gas2Die)
            }
            (EU::Diesel(volume_from), EU::Gasoline(volume_to)) => {
                Some(get_volume_factor(volume_from, volume_to)? * Die2Gas)
            }
            // Liquid equivalents non-gallons
            (EU::Gasoline(volume_from), EU::GallonsGasolineEquivalent) => {
                Some(get_volume_factor(volume_from, &V::GallonsUs)?)
            }
            (EU::Gasoline(volume_from), EU::GallonsDieselEquivalent) => {
                Some(get_volume_factor(volume_from, &V::GallonsUs)? * Gas2Die)
            }
            (EU::Diesel(volume_from), EU::GallonsDieselEquivalent) => {
                Some(get_volume_factor(volume_from, &V::GallonsUs)?)
            }
            (EU::Diesel(volume_from), EU::GallonsGasolineEquivalent) => {
                Some(get_volume_factor(volume_from, &V::GallonsUs)? * Die2Gas)
            }
            (EU::GallonsGasolineEquivalent, EU::Gasoline(volume_to)) => {
                Some(get_volume_factor(&V::GallonsUs, volume_to)?)
            }
            (EU::GallonsGasolineEquivalent, EU::Diesel(volume_to)) => {
                Some(get_volume_factor(&V::GallonsUs, volume_to)? * Gas2Die)
            }
            (EU::GallonsDieselEquivalent, EU::Diesel(volume_to)) => {
                Some(get_volume_factor(&V::GallonsUs, volume_to)?)
            }
            (EU::GallonsDieselEquivalent, EU::Gasoline(volume_to)) => {
                Some(get_volume_factor(&V::GallonsUs, volume_to)? * Die2Gas)
            }

            // Liquid to Non-liquid
            (EU::Gasoline(volume_from), EU::KilowattHours) => {
                Some(get_volume_factor(volume_from, &V::GallonsUs)? * Gas2Kwh)
            }
            (EU::Gasoline(volume_from), EU::KiloJoules) => {
                Some(get_volume_factor(volume_from, &V::GallonsUs)? * Gas2Kwh * kWh2Kj)
            }
            (EU::Gasoline(volume_from), EU::BTU) => {
                Some(get_volume_factor(volume_from, &V::GallonsUs)? * Gas2Kwh * kWh2BTU)
            }
            (EU::Diesel(volume_from), EU::KilowattHours) => {
                Some(get_volume_factor(volume_from, &V::GallonsUs)? * Die2Gas * Gas2Kwh)
            }
            (EU::Diesel(volume_from), EU::KiloJoules) => {
                Some(get_volume_factor(volume_from, &V::GallonsUs)? * Die2Gas * Gas2Kwh * kWh2Kj)
            }
            (EU::Diesel(volume_from), EU::BTU) => {
                Some(get_volume_factor(volume_from, &V::GallonsUs)? * Die2Gas * Gas2Kwh * kWh2BTU)
            }

            // Non-liquid to Liquid
            (EU::KilowattHours, EU::Gasoline(volume_to)) => {
                Some(kWh2Gas * get_volume_factor(&V::GallonsUs, volume_to)?)
            }
            (EU::KiloJoules, EU::Gasoline(volume_to)) => {
                Some(Kj2Kwh * kWh2Gas * get_volume_factor(&V::GallonsUs, volume_to)?)
            }
            (EU::BTU, EU::Gasoline(volume_to)) => {
                Some(BTU2Kwh * kWh2Gas * get_volume_factor(&V::GallonsUs, volume_to)?)
            }
            (EU::KilowattHours, EU::Diesel(volume_to)) => {
                Some(kWh2Gas * Gas2Die * get_volume_factor(&V::GallonsUs, volume_to)?)
            }
            (EU::KiloJoules, EU::Diesel(volume_to)) => {
                Some(Kj2Kwh * kWh2Gas * Gas2Die * get_volume_factor(&V::GallonsUs, volume_to)?)
            }
            (EU::BTU, EU::Diesel(volume_to)) => {
                Some(BTU2Kwh * kWh2Gas * Gas2Die * get_volume_factor(&V::GallonsUs, volume_to)?)
            }

            // Non-liquid to Non-liquid
            (EU::KilowattHours, EU::KiloJoules) => Some(kWh2Kj),
            (EU::KilowattHours, EU::BTU) => Some(kWh2BTU),
            (EU::KilowattHours, EU::GallonsGasolineEquivalent) => Some(kWh2Gas),
            (EU::KilowattHours, EU::GallonsDieselEquivalent) => Some(kWh2Gas * Gas2Die),
            (EU::KiloJoules, EU::KilowattHours) => Some(Kj2Kwh),
            (EU::KiloJoules, EU::BTU) => Some(Kj2Kwh * kWh2BTU),
            (EU::KiloJoules, EU::GallonsGasolineEquivalent) => Some(Kj2Kwh * kWh2Gas),
            (EU::KiloJoules, EU::GallonsDieselEquivalent) => Some(Kj2Kwh * kWh2Gas * Gas2Die),
            (EU::BTU, EU::KilowattHours) => Some(BTU2Kwh),
            (EU::BTU, EU::KiloJoules) => Some(BTU2Kwh * kWh2Kj),
            (EU::BTU, EU::GallonsGasolineEquivalent) => Some(BTU2Kwh * kWh2Gas),
            (EU::BTU, EU::GallonsDieselEquivalent) => Some(BTU2Kwh * kWh2Gas * Gas2Die),
            (EU::GallonsGasolineEquivalent, EU::KilowattHours) => Some(Gas2Kwh),
            (EU::GallonsGasolineEquivalent, EU::KiloJoules) => Some(Gas2Kwh * kWh2Kj),
            (EU::GallonsGasolineEquivalent, EU::BTU) => Some(Gas2Kwh * kWh2BTU),
            (EU::GallonsGasolineEquivalent, EU::GallonsDieselEquivalent) => Some(Gas2Die),
            (EU::GallonsDieselEquivalent, EU::KilowattHours) => Some(Die2Gas * Gas2Kwh),
            (EU::GallonsDieselEquivalent, EU::KiloJoules) => Some(Die2Gas * Gas2Kwh * kWh2Kj),
            (EU::GallonsDieselEquivalent, EU::BTU) => Some(Die2Gas * Gas2Kwh * kWh2BTU),
            (EU::GallonsDieselEquivalent, EU::GallonsGasolineEquivalent) => Some(Die2Gas),

            // This arm was needed to keep the compiler happy about the (x, y) if x == y arm
            _ => None,
        };

        if let Some(factor) = conversion_factor {
            let updated = Energy::from(value.as_ref().as_f64() * factor);
            *value.to_mut() = updated;
        }
        Ok(())
    }

    fn convert_to_base(&self, value: &mut std::borrow::Cow<Energy>) -> Result<(), UnitError> {
        self.convert(value, &baseunit::ENERGY_UNIT)
    }
}

impl std::fmt::Display for EnergyUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_string(self)
            .map_err(|_| std::fmt::Error)?
            .replace('\"', "");
        write!(f, "{}", s)
    }
}

impl FromStr for EnergyUnit {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use EnergyUnit as E;
        match s
            .trim()
            .to_lowercase()
            .replace("_", " ")
            .replace(" ", "")
            .as_str()
        {
            "gallonsgasoline" => Ok(E::Gasoline(VolumeUnit::GallonsUs)),
            "gallonsdiesel" => Ok(E::Diesel(VolumeUnit::GallonsUs)),
            "ukgallonsgasoline" => Ok(E::Gasoline(VolumeUnit::GallonsUk)),
            "ukgallonsdiesel" => Ok(E::Diesel(VolumeUnit::GallonsUk)),
            "kilowatthours" | "kilowatthour" | "kwh" => Ok(E::KilowattHours),
            "litersgasoline" => Ok(E::Gasoline(VolumeUnit::Liters)),
            "litersdiesel" => Ok(E::Diesel(VolumeUnit::Liters)),
            "gallonsgasolineequivalent" | "gge" => Ok(E::GallonsGasolineEquivalent),
            "kilojoules" | "kj" => Ok(E::KiloJoules),
            "btu" | "britishthermalunit" => Ok(E::BTU),
            _ => Err(format!("unknown energy unit '{}'", s)),
        }
    }
}

impl TryFrom<String> for EnergyUnit {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}
#[cfg(test)]
mod test {
    use crate::model::unit::internal_float::InternalFloat;
    use crate::model::unit::Energy;
    use crate::model::unit::EnergyUnit as U;
    use crate::model::unit::VolumeUnit as V;
    use crate::model::unit::{AsF64, Convert};
    /// The logic behind these test is to manually compute
    /// conversion factors from kWh to all other units.
    /// We then run two tests:
    ///     - All these conversion factors match the `EnergyUnit::kWh::convert` implementation
    ///     - All pairs of `EnergyUnit` variants match the implementation
    /// We implement the second point by computing A->B from the manually computed constants as
    ///     A -> kWh x kWh -> B
    /// and compare that to the value implemented in `convert`
    ///
    /// One shortcoming of this approach is that the tests short-circuit if one fails. This could
    /// be fixed with a macro or a crate to generate tests
    use std::borrow::Cow;

    const UNIT_VARIANTS: [U; 11] = [
        U::KilowattHours,
        U::KiloJoules,
        U::BTU,
        U::Gasoline(V::GallonsUs),
        U::Gasoline(V::GallonsUk),
        U::Gasoline(V::Liters),
        U::Diesel(V::GallonsUs),
        U::Diesel(V::GallonsUk),
        U::Diesel(V::Liters),
        U::GallonsGasolineEquivalent,
        U::GallonsDieselEquivalent,
    ];

    // Manually compute kWh -> * factors
    fn kwh_factors(to: U) -> f64 {
        match to {
            U::KilowattHours => 1.,
            U::Gasoline(V::GallonsUs) => 1. / 33.41,
            U::Gasoline(V::GallonsUk) => 0.832674 / 33.41,
            U::Gasoline(V::Liters) => 3.78541 / 33.41,
            U::Diesel(V::GallonsUs) => 1. / 37.95,
            U::Diesel(V::GallonsUk) => 0.832674 / 37.95,
            U::Diesel(V::Liters) => 3.78541 / 37.95,
            U::GallonsGasolineEquivalent => 1. / 33.41,
            U::GallonsDieselEquivalent => 1. / 37.95,
            U::KiloJoules => 3600.,
            U::BTU => 3412.14,
        }
    }

    fn assert_approx_eq(a: Energy, b: Energy, error: f64, unit_a: U, unit_b: U) {
        // We are checking for relative error so `a` should be large enough
        assert!(
            (*a > InternalFloat::MIN) || (*a < -InternalFloat::MIN),
            "Cannot test relative error {} ~= {}: Value {} is too close to zero",
            a,
            b,
            a
        );

        let abs_diff = match (a, b) {
            (c, d) if c < d => (d - c).as_f64(),
            (c, d) if c >= d => (c - d).as_f64(),
            (_, _) => 0.,
        };
        let relative_error = abs_diff / a.as_f64();
        assert!(
            relative_error < error,
            "{:?} -> {:?}: {} ~= {} is not true within a relative error of {}",
            unit_a,
            unit_b,
            a,
            b,
            error
        )
    }

    #[test]
    fn test_base_values() {
        for to_unit in UNIT_VARIANTS {
            let mut value: Cow<'_, Energy> = Cow::Owned(Energy::ONE);
            let target = kwh_factors(to_unit);
            U::KilowattHours.convert(&mut value, &to_unit).unwrap();
            assert_approx_eq(
                value.into_owned(),
                Energy::from(target),
                0.001,
                U::KilowattHours,
                to_unit,
            );
        }
    }

    #[test]
    fn test_all_pairs() {
        for from_unit in UNIT_VARIANTS {
            for to_unit in UNIT_VARIANTS {
                // From -> kWh
                let from_factor = 1. / kwh_factors(from_unit);
                // kWh -> To
                let to_factor = kwh_factors(to_unit);
                let target = from_factor * to_factor;

                let mut value: Cow<'_, Energy> = Cow::Owned(Energy::ONE);
                from_unit.convert(&mut value, &to_unit).unwrap();
                assert_approx_eq(
                    value.into_owned(),
                    Energy::from(target),
                    0.001,
                    from_unit,
                    to_unit,
                );
            }
        }
    }
}
