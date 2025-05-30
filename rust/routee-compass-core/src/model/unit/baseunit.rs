use crate::model::unit::{
    DistanceUnit, EnergyRateUnit, EnergyUnit, GradeUnit, SpeedUnit, TimeUnit, WeightUnit,
};

use super::VolumeUnit;

/// RouteE Compass edges-compass.csv.gz files store distance in meters
pub const DISTANCE_UNIT: DistanceUnit = DistanceUnit::Meters;

pub const TIME_UNIT: TimeUnit = TimeUnit::Seconds;
pub const ENERGY_UNIT: EnergyUnit = EnergyUnit::KilowattHours;
pub const GRADE_UNIT: GradeUnit = GradeUnit::Decimal;
pub const VOLUME_UNIT: VolumeUnit = VolumeUnit::Liters;
pub const WEIGHT_UNIT: WeightUnit = WeightUnit::Kg;
pub const SPEED_UNIT: SpeedUnit = SpeedUnit(DISTANCE_UNIT, TIME_UNIT);
pub const ENERGY_RATE_UNIT: EnergyRateUnit =
    EnergyRateUnit::EnergyPerDistance(ENERGY_UNIT, DISTANCE_UNIT);
