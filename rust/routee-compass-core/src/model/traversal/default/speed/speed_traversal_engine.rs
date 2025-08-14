use crate::model::unit::SpeedUnit;
use crate::util::fs::read_decoders;
use crate::{model::traversal::TraversalModelError, util::fs::read_utils};
use kdam::Bar;
use std::path::Path;
use uom::si::f64::Velocity;
use uom::ConstZero;

pub struct SpeedTraversalEngine {
    pub speed_table: Box<[Velocity]>,
    pub max_speed: Velocity,
}

impl SpeedTraversalEngine {
    pub fn new<P: AsRef<Path>>(
        speed_table_path: &P,
        speed_unit: SpeedUnit,
    ) -> Result<SpeedTraversalEngine, TraversalModelError> {
        let speed_table: Box<[Velocity]> = read_utils::read_raw_file(
            speed_table_path,
            read_decoders::f64,
            Some(Bar::builder().desc("link speeds")),
            None,
        )
        .map_err(|e| {
            TraversalModelError::BuildError(format!(
                "cannot read {} due to {}",
                speed_table_path.as_ref().to_str().unwrap_or_default(),
                e,
            ))
        })?
        .iter()
        .map(|&s| speed_unit.to_uom(s))
        .collect::<Vec<Velocity>>()
        .into_boxed_slice();
        let max_speed = get_max_speed(&speed_table)?;
        let model = SpeedTraversalEngine {
            speed_table,
            max_speed,
        };
        Ok(model)
    }
}

pub fn get_max_speed(speed_table: &[Velocity]) -> Result<Velocity, TraversalModelError> {
    let (max_speed, count) =
        speed_table
            .iter()
            .fold((Velocity::ZERO, 0), |(acc_max, acc_cnt), row| {
                let next_max = if acc_max > *row { acc_max } else { *row };
                (next_max, acc_cnt + 1)
            });

    if count == 0 {
        let msg = format!("parsed {count} entries for speed table");
        Err(TraversalModelError::BuildError(msg))
    } else if max_speed == Velocity::ZERO {
        let msg = format!("max speed was zero in speed table with {count} entries");
        Err(TraversalModelError::BuildError(msg))
    } else {
        Ok(max_speed)
    }
}
