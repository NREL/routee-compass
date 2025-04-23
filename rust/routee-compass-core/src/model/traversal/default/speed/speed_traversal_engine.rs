use kdam::Bar;

use crate::model::unit::DistanceUnit;
use crate::model::unit::{baseunit, SpeedUnit, TimeUnit};
use crate::util::fs::read_decoders;
use crate::{
    model::{traversal::TraversalModelError, unit::Speed},
    util::fs::read_utils,
};
use std::path::Path;

pub struct SpeedTraversalEngine {
    pub speed_table: Box<[Speed]>,
    pub speed_unit: SpeedUnit,
    pub time_unit: TimeUnit,
    pub distance_unit: DistanceUnit,
    pub max_speed: Speed,
}

impl SpeedTraversalEngine {
    pub fn new<P: AsRef<Path>>(
        speed_table_path: &P,
        speed_unit: SpeedUnit,
        distance_unit_opt: Option<DistanceUnit>,
        time_unit_opt: Option<TimeUnit>,
    ) -> Result<SpeedTraversalEngine, TraversalModelError> {
        let speed_table: Box<[Speed]> = read_utils::read_raw_file(
            speed_table_path,
            read_decoders::default,
            Some(Bar::builder().desc("link speeds")),
            None,
        )
        .map_err(|e| {
            TraversalModelError::BuildError(format!(
                "cannot read {} due to {}",
                speed_table_path.as_ref().to_str().unwrap_or_default(),
                e,
            ))
        })?;
        let max_speed = get_max_speed(&speed_table)?;
        let time_unit = time_unit_opt.unwrap_or(baseunit::TIME_UNIT);
        let distance_unit = distance_unit_opt.unwrap_or(baseunit::DISTANCE_UNIT);
        let model = SpeedTraversalEngine {
            speed_table,
            distance_unit,
            time_unit,
            speed_unit,
            max_speed,
        };
        Ok(model)
    }
}

pub fn get_max_speed(speed_table: &[Speed]) -> Result<Speed, TraversalModelError> {
    let (max_speed, count) =
        speed_table
            .iter()
            .fold((Speed::ZERO, 0), |(acc_max, acc_cnt), row| {
                let next_max = if acc_max > *row { acc_max } else { *row };
                (next_max, acc_cnt + 1)
            });

    if count == 0 {
        let msg = format!("parsed {} entries for speed table", count);
        Err(TraversalModelError::BuildError(msg))
    } else if max_speed == Speed::ZERO {
        let msg = format!("max speed was zero in speed table with {} entries", count);
        Err(TraversalModelError::BuildError(msg))
    } else {
        Ok(max_speed)
    }
}
