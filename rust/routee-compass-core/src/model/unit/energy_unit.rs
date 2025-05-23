use super::{baseunit, Convert, Energy, UnitError, VolumeUnit};
use crate::model::unit::AsF64;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

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
    BTU
}

#[allow(non_upper_case_globals)]
impl Convert<Energy> for EnergyUnit {
    fn convert(&self, value: &mut std::borrow::Cow<Energy>, to: &Self) -> Result<(), UnitError> {
        use EnergyUnit as S;
        use VolumeUnit as V;

        const USgal2UKgal: f64 = 0.832674;
        const UKgal2USgal: f64 = 1.20095;
        const USgal2L: f64 = 3.78541;
        const L2USgal: f64 = 0.264172;
        const UKgal2L: f64 = UKgal2USgal * USgal2L;
        const L2UKgal: f64 = L2USgal * USgal2UKgal;

        const Gas2Die: f64 = 33.41 / 37.95;
        const Die2Gas: f64 = 37.95 / 33.41;
        const Gas2Kwh: f64 = 33.41;
        const kWh2Gas: f64 = 1. / 33.41;
        const kWh2Kj: f64 = 3_600.;
        const Kj2Kwh: f64 = 1. / 3600.; 
        const kWh2BTU: f64 = 3_412.14;
        const BTU2Kwh: f64 = 1. / 3412.14;

        let conversion_factor = match (self, to) {
            (S::Gasoline(V::GallonsUs), S::Gasoline(V::GallonsUs)) => None,
            (S::Gasoline(V::GallonsUs), S::GallonsGasolineEquivalent) => None,
            (S::Gasoline(V::GallonsUs), S::Gasoline(V::GallonsUk)) => Some(USgal2UKgal),
            (S::Gasoline(V::GallonsUs), S::Gasoline(V::Liters)) => Some(USgal2L),
            (S::Gasoline(V::GallonsUs), S::Diesel(V::GallonsUs)) => Some(Gas2Die),
            (S::Gasoline(V::GallonsUs), S::GallonsDieselEquivalent) => Some(Gas2Die),
            (S::Gasoline(V::GallonsUs), S::Diesel(V::GallonsUk)) => Some(USgal2UKgal * Gas2Die),
            (S::Gasoline(V::GallonsUs), S::Diesel(V::Liters)) => Some(USgal2L * Gas2Die),
            (S::Gasoline(V::GallonsUs), S::KilowattHours) => Some(Gas2Kwh),
            (S::Gasoline(V::GallonsUs), S::KiloJoules) => Some(Gas2Kwh * kWh2Kj),
            (S::Gasoline(V::GallonsUs), S::BTU) => Some(Gas2Kwh * kWh2BTU),

            (S::GallonsGasolineEquivalent, S::Gasoline(V::GallonsUs)) => None,
            (S::GallonsGasolineEquivalent, S::GallonsGasolineEquivalent) => None,
            (S::GallonsGasolineEquivalent, S::Gasoline(V::GallonsUk)) => Some(USgal2UKgal),
            (S::GallonsGasolineEquivalent, S::Gasoline(V::Liters)) => Some(USgal2L),
            (S::GallonsGasolineEquivalent, S::Diesel(V::GallonsUs)) => Some(Gas2Die),
            (S::GallonsGasolineEquivalent, S::GallonsDieselEquivalent) => Some(Gas2Die),
            (S::GallonsGasolineEquivalent, S::Diesel(V::GallonsUk)) => Some(USgal2UKgal * Gas2Die),
            (S::GallonsGasolineEquivalent, S::Diesel(V::Liters)) => Some(USgal2L * Gas2Die),
            (S::GallonsGasolineEquivalent, S::KilowattHours) => Some(Gas2Kwh),
            (S::GallonsGasolineEquivalent, S::KiloJoules) => Some(Gas2Kwh * kWh2Kj),
            (S::GallonsGasolineEquivalent, S::BTU) => Some(Gas2Kwh * kWh2BTU),
            
            (S::Gasoline(V::Liters), S::Gasoline(V::GallonsUs)) => Some(L2USgal),
            (S::Gasoline(V::Liters), S::GallonsGasolineEquivalent) => Some(L2USgal),
            (S::Gasoline(V::Liters), S::Gasoline(V::GallonsUk)) => Some(L2UKgal),
            (S::Gasoline(V::Liters), S::Gasoline(V::Liters)) => None,
            (S::Gasoline(V::Liters), S::Diesel(V::GallonsUs)) => Some(L2USgal * Gas2Die),
            (S::Gasoline(V::Liters), S::GallonsDieselEquivalent) => Some(L2USgal * Gas2Die),
            (S::Gasoline(V::Liters), S::Diesel(V::GallonsUk)) => Some(L2UKgal * Gas2Die),
            (S::Gasoline(V::Liters), S::Diesel(V::Liters)) => Some(Gas2Die),
            (S::Gasoline(V::Liters), S::KilowattHours) => Some(L2USgal * Gas2Kwh),
            (S::Gasoline(V::Liters), S::KiloJoules) => Some(L2USgal * Gas2Kwh * kWh2Kj),       
            (S::Gasoline(V::Liters), S::BTU) => Some(L2USgal * Gas2Kwh * kWh2BTU),       
            
            (S::Diesel(V::GallonsUs), S::Gasoline(V::GallonsUs)) => Some(Die2Gas),
            (S::Diesel(V::GallonsUs), S::GallonsGasolineEquivalent) => Some(Die2Gas),
            (S::Diesel(V::GallonsUs), S::Gasoline(V::GallonsUk)) => Some(USgal2UKgal * Die2Gas),
            (S::Diesel(V::GallonsUs), S::Gasoline(V::Liters)) => Some(USgal2L * Die2Gas),
            (S::Diesel(V::GallonsUs), S::Diesel(V::GallonsUs)) => None,
            (S::Diesel(V::GallonsUs), S::GallonsDieselEquivalent) => None,
            (S::Diesel(V::GallonsUs), S::Diesel(V::GallonsUk)) => Some(USgal2UKgal),
            (S::Diesel(V::GallonsUs), S::Diesel(V::Liters)) => Some(USgal2L),
            (S::Diesel(V::GallonsUs), S::KilowattHours) => Some(Die2Gas * Gas2Kwh),
            (S::Diesel(V::GallonsUs), S::KiloJoules) => Some(Die2Gas * Gas2Kwh * kWh2Kj),
            (S::Diesel(V::GallonsUs), S::BTU) => Some(Die2Gas * Gas2Kwh * kWh2BTU),

            (S::GallonsDieselEquivalent, S::Gasoline(V::GallonsUs)) => Some(Die2Gas),
            (S::GallonsDieselEquivalent, S::GallonsGasolineEquivalent) => Some(Die2Gas),
            (S::GallonsDieselEquivalent, S::Gasoline(V::GallonsUk)) => Some(USgal2UKgal * Die2Gas),
            (S::GallonsDieselEquivalent, S::Gasoline(V::Liters)) => Some(USgal2L * Die2Gas),
            (S::GallonsDieselEquivalent, S::Diesel(V::GallonsUs)) => None,
            (S::GallonsDieselEquivalent, S::GallonsDieselEquivalent) => None,
            (S::GallonsDieselEquivalent, S::Diesel(V::GallonsUk)) => Some(USgal2UKgal),
            (S::GallonsDieselEquivalent, S::Diesel(V::Liters)) => Some(USgal2L),
            (S::GallonsDieselEquivalent, S::KilowattHours) => Some(Die2Gas * Gas2Kwh),
            (S::GallonsDieselEquivalent, S::KiloJoules) => Some(Die2Gas * Gas2Kwh * kWh2Kj),
            (S::GallonsDieselEquivalent, S::BTU) => Some(Die2Gas * Gas2Kwh * kWh2BTU),
            
            (S::Diesel(V::Liters), S::Gasoline(V::GallonsUs)) => Some(L2USgal * Die2Gas),
            (S::Diesel(V::Liters), S::GallonsGasolineEquivalent) => Some(L2USgal * Die2Gas),
            (S::Diesel(V::Liters), S::Gasoline(V::GallonsUk)) => Some(L2UKgal * Die2Gas),
            (S::Diesel(V::Liters), S::Gasoline(V::Liters)) => Some(Die2Gas),
            (S::Diesel(V::Liters), S::Diesel(V::GallonsUs)) => Some(L2USgal),
            (S::Diesel(V::Liters), S::GallonsDieselEquivalent) => Some(L2USgal),
            (S::Diesel(V::Liters), S::Diesel(V::GallonsUk)) => Some(L2UKgal),
            (S::Diesel(V::Liters), S::Diesel(V::Liters)) => None,
            (S::Diesel(V::Liters), S::KilowattHours) => Some(L2USgal * Die2Gas * Gas2Kwh),
            (S::Diesel(V::Liters), S::KiloJoules) => Some(L2USgal * Die2Gas * Gas2Kwh * kWh2Kj),
            (S::Diesel(V::Liters), S::BTU) => Some(L2USgal * Die2Gas * Gas2Kwh * kWh2BTU),
            
            (S::KilowattHours, S::Gasoline(V::GallonsUs)) => Some(kWh2Gas),
            (S::KilowattHours, S::GallonsGasolineEquivalent) => Some(kWh2Gas),
            (S::KilowattHours, S::Gasoline(V::GallonsUk)) => Some(kWh2Gas * USgal2UKgal),
            (S::KilowattHours, S::Gasoline(V::Liters)) => Some(kWh2Gas * USgal2L),
            (S::KilowattHours, S::Diesel(V::GallonsUs)) => Some(kWh2Gas * Gas2Die),
            (S::KilowattHours, S::GallonsDieselEquivalent) => Some(kWh2Gas * Gas2Die),
            (S::KilowattHours, S::Diesel(V::GallonsUk)) => Some(kWh2Gas * Gas2Die * USgal2UKgal),
            (S::KilowattHours, S::Diesel(V::Liters)) => Some(kWh2Gas * Gas2Die * USgal2L),
            (S::KilowattHours, S::KilowattHours) => None,
            (S::KilowattHours, S::KiloJoules) => Some(kWh2Kj),
            (S::KilowattHours, S::BTU) => Some(kWh2BTU),
            
            (S::Gasoline(V::GallonsUk), S::Gasoline(V::GallonsUs)) => Some(UKgal2USgal),
            (S::Gasoline(V::GallonsUk), S::GallonsGasolineEquivalent) => Some(UKgal2USgal),
            (S::Gasoline(V::GallonsUk), S::Gasoline(V::GallonsUk)) => None,
            (S::Gasoline(V::GallonsUk), S::Gasoline(V::Liters)) => Some(UKgal2L),
            (S::Gasoline(V::GallonsUk), S::Diesel(V::GallonsUs)) => Some(UKgal2USgal * Gas2Die),
            (S::Gasoline(V::GallonsUk), S::GallonsDieselEquivalent) => Some(UKgal2USgal * Gas2Die),
            (S::Gasoline(V::GallonsUk), S::Diesel(V::GallonsUk)) => Some(Gas2Die),
            (S::Gasoline(V::GallonsUk), S::Diesel(V::Liters)) => Some(UKgal2L * Gas2Die),
            (S::Gasoline(V::GallonsUk), S::KilowattHours) => Some(UKgal2USgal * Gas2Kwh),
            (S::Gasoline(V::GallonsUk), S::KiloJoules) => Some(UKgal2USgal * Gas2Kwh * kWh2Kj),
            (S::Gasoline(V::GallonsUk), S::BTU) => Some(UKgal2USgal * Gas2Kwh * kWh2BTU),
            
            (S::Diesel(V::GallonsUk), S::Gasoline(V::GallonsUs)) => Some(UKgal2USgal * Die2Gas),
            (S::Diesel(V::GallonsUk), S::GallonsGasolineEquivalent) => Some(UKgal2USgal * Die2Gas),
            (S::Diesel(V::GallonsUk), S::Gasoline(V::GallonsUk)) => Some(Die2Gas),
            (S::Diesel(V::GallonsUk), S::Gasoline(V::Liters)) => Some(UKgal2L * Die2Gas),
            (S::Diesel(V::GallonsUk), S::Diesel(V::GallonsUs)) => Some(UKgal2USgal),
            (S::Diesel(V::GallonsUk), S::GallonsDieselEquivalent) => Some(UKgal2USgal),
            (S::Diesel(V::GallonsUk), S::Diesel(V::GallonsUk)) => None,
            (S::Diesel(V::GallonsUk), S::Diesel(V::Liters)) => Some(UKgal2L),
            (S::Diesel(V::GallonsUk), S::KilowattHours) => Some(UKgal2USgal * Die2Gas * Gas2Kwh),
            (S::Diesel(V::GallonsUk), S::KiloJoules) => Some(UKgal2USgal * Die2Gas * Gas2Kwh * kWh2Kj),
            (S::Diesel(V::GallonsUk), S::BTU) => Some(UKgal2USgal * Die2Gas * Gas2Kwh * kWh2BTU),

            (S::KiloJoules, S::Gasoline(V::GallonsUs)) => Some(Kj2Kwh * kWh2Gas),
            (S::KiloJoules, S::GallonsGasolineEquivalent) => Some(Kj2Kwh * kWh2Gas),
            (S::KiloJoules, S::Gasoline(V::GallonsUk)) => Some(Kj2Kwh * kWh2Gas * USgal2UKgal),
            (S::KiloJoules, S::Gasoline(V::Liters)) => Some(Kj2Kwh * kWh2Gas * USgal2L),
            (S::KiloJoules, S::Diesel(V::GallonsUs)) => Some(Kj2Kwh * kWh2Gas * Gas2Die),
            (S::KiloJoules, S::GallonsDieselEquivalent) => Some(Kj2Kwh * kWh2Gas * Gas2Die),
            (S::KiloJoules, S::Diesel(V::GallonsUk)) => Some(Kj2Kwh * kWh2Gas * Gas2Die * USgal2UKgal),
            (S::KiloJoules, S::Diesel(V::Liters)) => Some(Kj2Kwh * kWh2Gas * Gas2Die * USgal2L),
            (S::KiloJoules, S::KilowattHours) => Some(Kj2Kwh),
            (S::KiloJoules, S::KiloJoules) => None,
            (S::KiloJoules, S::BTU) => Some(Kj2Kwh * kWh2BTU),

            (S::BTU, S::Gasoline(V::GallonsUs)) => Some(BTU2Kwh * kWh2Gas),
            (S::BTU, S::GallonsGasolineEquivalent) => Some(BTU2Kwh * kWh2Gas),
            (S::BTU, S::Gasoline(V::GallonsUk)) => Some(BTU2Kwh * kWh2Gas * USgal2UKgal),
            (S::BTU, S::Gasoline(V::Liters)) => Some(BTU2Kwh * kWh2Gas * USgal2L),
            (S::BTU, S::Diesel(V::GallonsUs)) => Some(BTU2Kwh * kWh2Gas * Gas2Die),
            (S::BTU, S::GallonsDieselEquivalent) => Some(BTU2Kwh * kWh2Gas * Gas2Die),
            (S::BTU, S::Diesel(V::GallonsUk)) => Some(BTU2Kwh * kWh2Gas * Gas2Die * USgal2UKgal),
            (S::BTU, S::Diesel(V::Liters)) => Some(BTU2Kwh * kWh2Gas * Gas2Die * USgal2L),
            (S::BTU, S::KilowattHours) => Some(BTU2Kwh),
            (S::BTU, S::KiloJoules) => Some(BTU2Kwh * kWh2Kj),
            (S::BTU, S::BTU) => None,
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
mod test{
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
    use crate::model::unit::Energy;
    use crate::model::unit::{AsF64, Convert};
    use crate::model::unit::EnergyUnit as U;
    use crate::model::unit::VolumeUnit as V;
    
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
        U::GallonsDieselEquivalent
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
        const ZERO_TOLERANCE: f64 = 0.00000001;
        assert!((a.as_f64() > ZERO_TOLERANCE) || (a.as_f64() < -ZERO_TOLERANCE), "Cannot test relative error {} ~= {}: Value {} is too close to zero", a, b, a);
        
        let abs_diff = match (a, b) {
            (c, d) if c < d => (d - c).as_f64(),
            (c, d) if c >= d => (c - d).as_f64(),
            (_, _) => 0.
        };
        let relative_error = abs_diff / a.as_f64();
        assert!(
            relative_error < error,
            "{:?} -> {:?}: {} ~= {} is not true within a relative error of {}",
            unit_a, unit_b, a, b, error
        )
    }

    #[test]
    fn test_base_values(){
        for to_unit in UNIT_VARIANTS{
            let mut value: Cow<'_, Energy> = Cow::Owned(Energy::ONE);
            let target = kwh_factors(to_unit);
            U::KilowattHours.convert(&mut value, &to_unit).unwrap();
            assert_approx_eq(value.into_owned(), Energy::from(target), 0.001, U::KilowattHours, to_unit);
        }
    }

    #[test]
    fn test_all_pairs(){
        for from_unit in UNIT_VARIANTS{
            for to_unit in UNIT_VARIANTS{
                // From -> kWh
                let from_factor = 1. / kwh_factors(from_unit);
                // kWh -> To
                let to_factor = kwh_factors(to_unit);
                let target = from_factor * to_factor;
                
                let mut value: Cow<'_, Energy> = Cow::Owned(Energy::ONE);
                from_unit.convert(&mut value, &to_unit).unwrap();
                assert_approx_eq(value.into_owned(), Energy::from(target), 0.001, from_unit, to_unit);
            }
        }
    }

}