use crate::model::unit::{
    DistanceUnit, EnergyRateUnit, EnergyUnit, GradeUnit, SpeedUnit, TimeUnit, WeightUnit,
};

pub const DISTANCE_UNIT: DistanceUnit = DistanceUnit::Meters;
pub const TIME_UNIT: TimeUnit = TimeUnit::Seconds;
pub const ENERGY_UNIT: EnergyUnit = EnergyUnit::KilowattHours;
pub const GRADE_UNIT: GradeUnit = GradeUnit::Decimal;
pub const WEIGHT_UNIT: WeightUnit = WeightUnit::Kg;
pub const SPEED_UNIT: SpeedUnit = SpeedUnit(DISTANCE_UNIT, TIME_UNIT);
pub const ENERGY_RATE_UNIT: EnergyRateUnit =
    EnergyRateUnit::EnergyPerDistance(ENERGY_UNIT, DISTANCE_UNIT);
