use super::{
    Distance, DistanceUnit, Energy, EnergyRate, EnergyRateUnit, EnergyUnit, Speed, SpeedUnit, Time,
    TimeUnit, UnitError,
};

pub const BASE_DISTANCE_UNIT: DistanceUnit = DistanceUnit::Meters;
pub const BASE_TIME_UNIT: TimeUnit = TimeUnit::Seconds;
pub const BASE_SPEED_UNIT: SpeedUnit = SpeedUnit::MetersPerSecond;

// these functions are accessible via the associated unit namespace. they are
// implemented here as they are coupled with the base units listed above.
// - Time::create_time()
// - Speed::create_speed()
// - Energy::create_energy()

/// calculates a time value based on the TimeUnit and incoming speed/distance values
/// in their unit types. First converts both Speed and Distance values to the Compass
/// base units. performs the division operation to get time and converts to the target
/// time unit.
pub fn create_time(
    speed: Speed,
    speed_unit: SpeedUnit,
    distance: Distance,
    distance_unit: DistanceUnit,
    time_unit: TimeUnit,
) -> Result<Time, UnitError> {
    let d = distance_unit.convert(distance, BASE_DISTANCE_UNIT);
    let s = speed_unit.convert(speed, BASE_SPEED_UNIT);
    if s <= Speed::ZERO || d <= Distance::ZERO {
        Err(UnitError::TimeFromSpeedAndDistanceError(
            speed,
            speed_unit,
            distance,
            distance_unit,
        ))
    } else {
        let time = (d, s).into();
        let result = BASE_TIME_UNIT.convert(time, time_unit);
        Ok(result)
    }
}

/// calculates a speed value based on the SpeedUnit and incoming time/distance values
/// in their unit types. First converts both Time and Distance values to the Compass
/// base units. performs the division operation to get speed and converts to the target
/// speed unit.
pub fn create_speed(
    time: Time,
    time_unit: TimeUnit,
    distance: Distance,
    distance_unit: DistanceUnit,
    speed_unit: SpeedUnit,
) -> Result<Speed, UnitError> {
    let d = distance_unit.convert(distance, BASE_DISTANCE_UNIT);
    let t = time_unit.convert(time, BASE_TIME_UNIT);
    if t <= Time::ZERO {
        Err(UnitError::SpeedFromTimeAndDistanceError(time, distance))
    } else {
        let speed = (d, t).into();
        let result = BASE_SPEED_UNIT.convert(speed, speed_unit);
        Ok(result)
    }
}

/// calculates an energy value based on some energy rate and distance.
/// the resulting energy unit is based on the energy rate unit provided.
pub fn create_energy(
    energy_rate: EnergyRate,
    energy_rate_unit: EnergyRateUnit,
    distance: Distance,
    distance_unit: DistanceUnit,
) -> Result<(Energy, EnergyUnit), UnitError> {
    // we don't make a conversion to a base energy unit here, since that's non-sensical for some unit types
    // instead, we rely on the associated units to direct our calculation.
    let rate_distance_unit = energy_rate_unit.associated_distance_unit();
    let energy_unit = energy_rate_unit.associated_energy_unit();
    let calc_distance = distance_unit.convert(distance, rate_distance_unit);
    let energy = (energy_rate, calc_distance).into();
    Ok((energy, energy_unit))
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::util::unit::as_f64::AsF64;

    fn approx_eq_time(a: Time, b: Time, error: f64) {
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

    fn approx_eq_speed(a: Speed, b: Speed, error: f64) {
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
    fn test_speed_calculate_fails() {
        let failure = create_speed(
            Time::ZERO,
            TimeUnit::Seconds,
            Distance::ONE,
            DistanceUnit::Meters,
            SpeedUnit::MetersPerSecond,
        );
        assert!(failure.is_err());
    }

    #[test]
    fn test_speed_calculate_idempotent() {
        let one_mps = create_speed(
            Time::ONE,
            TimeUnit::Seconds,
            Distance::ONE,
            DistanceUnit::Meters,
            SpeedUnit::MetersPerSecond,
        )
        .unwrap();
        assert_eq!(Speed::ONE, one_mps);
    }

    #[test]
    fn test_speed_calculate_imperial_to_si() {
        let speed_kph = create_speed(
            Time::ONE,
            TimeUnit::Hours,
            Distance::ONE,
            DistanceUnit::Miles,
            SpeedUnit::KilometersPerHour,
        )
        .unwrap();
        approx_eq_speed(Speed::new(1.60934), speed_kph, 0.001);
    }

    #[test]
    fn test_speed_calculate_kph_to_base() {
        let speed_kph = create_speed(
            Time::ONE,
            TimeUnit::Hours,
            Distance::ONE,
            DistanceUnit::Kilometers,
            BASE_SPEED_UNIT,
        )
        .unwrap();
        let expected = SpeedUnit::KilometersPerHour.convert(Speed::ONE, BASE_SPEED_UNIT);
        approx_eq_speed(speed_kph, expected, 0.001);
    }

    #[test]
    fn test_speed_calculate_base_to_kph() {
        let speed_kph = create_speed(
            Time::ONE,
            BASE_TIME_UNIT,
            Distance::ONE,
            BASE_DISTANCE_UNIT,
            SpeedUnit::KilometersPerHour,
        )
        .unwrap();
        let expected = SpeedUnit::MetersPerSecond.convert(Speed::ONE, SpeedUnit::KilometersPerHour);
        approx_eq_speed(speed_kph, expected, 0.001);
    }

    #[test]
    fn test_speed_calculate_mph_to_base() {
        let speed_kph = create_speed(
            Time::ONE,
            TimeUnit::Hours,
            Distance::ONE,
            DistanceUnit::Miles,
            BASE_SPEED_UNIT,
        )
        .unwrap();
        let expected = SpeedUnit::MilesPerHour.convert(Speed::ONE, BASE_SPEED_UNIT);
        approx_eq_speed(speed_kph, expected, 0.001);
    }

    #[test]
    fn test_speed_calculate_base_to_mph() {
        let speed_kph = create_speed(
            Time::ONE,
            BASE_TIME_UNIT,
            Distance::ONE,
            BASE_DISTANCE_UNIT,
            SpeedUnit::MilesPerHour,
        )
        .unwrap();
        let expected = SpeedUnit::MetersPerSecond.convert(Speed::ONE, SpeedUnit::MilesPerHour);
        approx_eq_speed(speed_kph, expected, 0.001);
    }

    #[test]
    fn test_time_calculate_fails() {
        let failure = create_time(
            Speed::ZERO,
            SpeedUnit::KilometersPerHour,
            Distance::ONE,
            DistanceUnit::Meters,
            BASE_TIME_UNIT,
        );
        assert!(failure.is_err());
    }

    #[test]
    fn test_time_calculate_idempotent() {
        let one_sec = create_time(
            Speed::ONE,
            SpeedUnit::MetersPerSecond,
            Distance::ONE,
            DistanceUnit::Meters,
            BASE_TIME_UNIT,
        )
        .unwrap();
        assert_eq!(Time::ONE, one_sec);
    }

    #[test]
    fn test_time_calculate_kph_to_base() {
        let time = create_time(
            Speed::ONE,
            SpeedUnit::KilometersPerHour,
            Distance::ONE,
            DistanceUnit::Kilometers,
            BASE_TIME_UNIT,
        )
        .unwrap();
        let expected = TimeUnit::Hours.convert(Time::ONE, BASE_TIME_UNIT);
        approx_eq_time(time, expected, 0.001);
    }

    #[test]
    fn test_time_calculate_base_to_kph() {
        let time = create_time(
            Speed::ONE,
            BASE_SPEED_UNIT,
            Distance::ONE,
            BASE_DISTANCE_UNIT,
            TimeUnit::Hours,
        )
        .unwrap();
        let expected = BASE_TIME_UNIT.convert(Time::ONE, TimeUnit::Hours);
        approx_eq_time(time, expected, 0.001);
    }

    #[test]
    fn test_time_calculate_mph_to_base() {
        let time = create_time(
            Speed::ONE,
            SpeedUnit::MilesPerHour,
            Distance::ONE,
            DistanceUnit::Miles,
            BASE_TIME_UNIT,
        )
        .unwrap();
        let expected = TimeUnit::Hours.convert(Time::ONE, BASE_TIME_UNIT);
        approx_eq_time(time, expected, 0.01);
    }

    #[test]
    fn test_time_calculate_base_to_mph() {
        let time = create_time(
            Speed::ONE,
            BASE_SPEED_UNIT,
            Distance::ONE,
            BASE_DISTANCE_UNIT,
            TimeUnit::Hours,
        )
        .unwrap();
        let expected = BASE_TIME_UNIT.convert(Time::ONE, TimeUnit::Hours);
        approx_eq_time(time, expected, 0.001);
    }

    #[test]
    fn test_energy_ggpm_meters() {
        let ten_mpg_rate = 1.0 / 10.0;
        let (energy, energy_unit) = create_energy(
            EnergyRate::new(ten_mpg_rate),
            EnergyRateUnit::GallonsGasolinePerMile,
            Distance::new(1609.0),
            DistanceUnit::Meters,
        )
        .unwrap();
        approx_eq_energy(energy, Energy::new(ten_mpg_rate), 0.00001);
        assert_eq!(energy_unit, EnergyUnit::GallonsGasoline);
    }

    #[test]
    fn test_energy_ggpm_miles() {
        let ten_mpg_rate = 1.0 / 10.0;
        let (energy, energy_unit) = create_energy(
            EnergyRate::new(ten_mpg_rate),
            EnergyRateUnit::GallonsGasolinePerMile,
            Distance::new(1.0),
            DistanceUnit::Miles,
        )
        .unwrap();
        approx_eq_energy(energy, Energy::new(ten_mpg_rate), 0.00001);
        assert_eq!(energy_unit, EnergyUnit::GallonsGasoline);
    }
}
