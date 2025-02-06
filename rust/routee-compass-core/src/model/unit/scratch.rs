use derive_more::{Add, Div, From, FromStr, Mul, Sub};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap, fmt::Display};

#[derive(
    Serialize,
    Deserialize,
    Clone,
    Debug,
    Default,
    Add,
    Div,
    Mul,
    Sub,
    From,
    FromStr,
    derive_more::derive::AsRef,
)]
struct Value(f64);

#[derive(Serialize, Deserialize)]
enum DistanceUnit {
    Feet,
    Meters,
}

#[derive(Serialize, Deserialize)]
enum TimeUnit {
    Seconds,
    Minutes,
}

#[derive(Serialize, Deserialize)]
enum LiquidFuelType {
    Gasoline,
    Diesel,
}

#[derive(Serialize, Deserialize)]
enum EnergyUnit {
    KilowattHours,
    Gallons(LiquidFuelType),
    Liters(LiquidFuelType),
}

#[derive(Serialize, Deserialize)]
enum UnitType {
    Distance(DistanceUnit),
    Time(TimeUnit),
    Energy(EnergyUnit),
}

trait Convert<T: Clone> {
    fn convert(&self, value: &mut Cow<T>, other: &Self) -> Result<(), String>;
}

impl Display for UnitType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnitType::Distance(_) => write!(f, "distance"),
            UnitType::Time(_) => write!(f, "time"),
            UnitType::Energy(_) => write!(f, "energy"),
        }
    }
}

impl Convert<Value> for DistanceUnit {
    fn convert(&self, value: &mut Cow<Value>, other: &Self) -> Result<(), String> {
        let conversion_factor = match (self, other) {
            (DistanceUnit::Feet, DistanceUnit::Meters) => Some(3.280839895),
            (DistanceUnit::Meters, DistanceUnit::Feet) => Some(0.3048),
            _ => None,
        };
        if let Some(factor) = conversion_factor {
            apply_conversion(value, factor);
        }
        Ok(())
    }
}

impl Convert<Value> for TimeUnit {
    fn convert(&self, value: &mut Cow<Value>, other: &Self) -> Result<(), String> {
        let conversion_factor = match (self, other) {
            (TimeUnit::Seconds, TimeUnit::Minutes) => todo!(),
            (TimeUnit::Minutes, TimeUnit::Seconds) => todo!(),
            _ => None,
        };
        if let Some(factor) = conversion_factor {
            apply_conversion(value, factor);
        }
        Ok(())
    }
}

impl Convert<Value> for EnergyUnit {
    fn convert(&self, value: &mut Cow<Value>, other: &Self) -> Result<(), String> {
        use LiquidFuelType::{Diesel, Gasoline};
        let conversion_factor = match (self, other) {
            (EnergyUnit::KilowattHours, EnergyUnit::Gallons(Gasoline)) => todo!(),
            (EnergyUnit::KilowattHours, EnergyUnit::Gallons(Diesel)) => todo!(),
            (EnergyUnit::Gallons(Gasoline), EnergyUnit::KilowattHours) => todo!(),
            (EnergyUnit::Gallons(Gasoline), EnergyUnit::Gallons(Diesel)) => todo!(),
            (EnergyUnit::Gallons(Diesel), EnergyUnit::KilowattHours) => todo!(),
            (EnergyUnit::Gallons(Diesel), EnergyUnit::Gallons(Gasoline)) => todo!(),
            (EnergyUnit::KilowattHours, EnergyUnit::Liters(Gasoline)) => todo!(),
            (EnergyUnit::KilowattHours, EnergyUnit::Liters(Diesel)) => todo!(),
            (EnergyUnit::Liters(Gasoline), EnergyUnit::KilowattHours) => todo!(),
            (EnergyUnit::Liters(Gasoline), EnergyUnit::Liters(Diesel)) => todo!(),
            (EnergyUnit::Liters(Diesel), EnergyUnit::KilowattHours) => todo!(),
            (EnergyUnit::Liters(Diesel), EnergyUnit::Liters(Gasoline)) => todo!(),
            _ => None,
        };
        if let Some(factor) = conversion_factor {
            apply_conversion(value, factor);
        }
        Ok(())
    }
}

impl Convert<Value> for UnitType {
    fn convert(&self, value: &mut Cow<Value>, other: &Self) -> Result<(), String> {
        match (self, other) {
            (UnitType::Distance(o), UnitType::Distance(d)) => o.convert(value, d),
            (UnitType::Time(o), UnitType::Time(d)) => o.convert(value, d),
            (UnitType::Energy(o), UnitType::Energy(d)) => o.convert(value, d),
            _ => Err(format!(
                "invalid UnitTypes for conversion: {} -> {}",
                self, other
            )),
        }
    }
}

fn apply_conversion(value: &mut Cow<Value>, factor: f64) {
    let mut updated = Value(value.0 * factor);
    let value_mut = value.to_mut();
    std::mem::swap(value_mut, &mut updated);
}

struct ValueState<'a> {
    value: Value,
    unit: &'a UnitType,
}

struct StateModel {
    indices: HashMap<String, usize>,
    units: HashMap<String, UnitType>,
}

impl StateModel {
    fn get<'a>(&'a self, name: &str, state: &[f64]) -> Result<ValueState<'a>, String> {
        let index = self
            .indices
            .get(name)
            .ok_or_else(|| format!("unknown feature {}", name))?;
        let unit = self
            .units
            .get(name)
            .ok_or_else(|| format!("unknown feature {}", name))?;
        let value = state
            .get(*index)
            .ok_or_else(|| format!("bad index {}", index))?;
        Ok(ValueState {
            value: Value(*value),
            unit,
        })
    }
}

#[cfg(test)]
mod test {

    use super::*;

    fn test_distance() {
        // setup
        let meters = UnitType::Distance(DistanceUnit::Meters);
        let model = StateModel {
            indices: HashMap::from([(String::from("distance"), 0)]),
            units: HashMap::from([(
                String::from("distance"),
                UnitType::Distance(DistanceUnit::Feet),
            )]),
        };
        let state_vector = vec![100];

        // use the state model to get the distance value, and then convert it from feet to meters
        let distance_state = model.get("distance", state_vector).unwrap();
        let mut distance = Cow::Borrowed(&distance_state.value);
        distance_state.unit.convert(&mut distance, &meters).unwrap();

        // 100 feet should equal 30.48 meters
        assert_eq!(distance.into_owned(), Value(100.0 * 0.3048))
    }
}
