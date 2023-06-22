use std::collections::HashMap;

use pyo3::prelude::*;

use crate::graph::Link;

pub type ProfileId = u16;
pub type SecondOfDay = u32;
pub type RelativeSpeed = f64;
pub type DayOfWeek = usize;

#[derive(Clone, Debug)]
pub struct SpeedModifier {
    pub second_of_day: SecondOfDay,
    pub relative_speed: RelativeSpeed,
}

#[pyclass]
#[derive(Clone, Debug)]
pub struct TimeOfDaySpeeds {
    pub speeds_modifiers: HashMap<ProfileId, Vec<SpeedModifier>>,
}

impl Default for TimeOfDaySpeeds {
    fn default() -> Self {
        TimeOfDaySpeeds {
            speeds_modifiers: HashMap::new(),
        }
    }
}

#[pymethods]
impl TimeOfDaySpeeds {
    #[new]
    pub fn new(
        mut speeds_modifiers: HashMap<ProfileId, Vec<(SecondOfDay, RelativeSpeed)>>,
    ) -> Self {
        // sort inputs by second of day and build up SpeedModifier structs
        for (_, speed_modifiers) in speeds_modifiers.iter_mut() {
            speed_modifiers.sort_by_key(|(second_of_day, _)| *second_of_day);
        }
        let speeds_modifiers = speeds_modifiers
            .into_iter()
            .map(|(profile_id, speed_modifiers)| {
                (
                    profile_id,
                    speed_modifiers
                        .into_iter()
                        .map(|(second_of_day, relative_speed)| SpeedModifier {
                            second_of_day,
                            relative_speed,
                        })
                        .collect(),
                )
            })
            .collect();

        TimeOfDaySpeeds { speeds_modifiers }
    }

    pub fn get_modifier_by_second_of_day(
        &self,
        profile_id: ProfileId,
        second_of_day: SecondOfDay,
    ) -> RelativeSpeed {
        // set a default speed modifier of 1.0 (no change in speed)
        let mut modifier = 1.0;

        if let Some(speed_modifiers) = self.speeds_modifiers.get(&profile_id) {
            // use a binary search to find where the second of day fits in the list of speed modifiers
            match speed_modifiers.binary_search_by(|sm| sm.second_of_day.cmp(&second_of_day)) {
                // if the second of day is found, use that speed modifier
                Ok(index) => modifier = speed_modifiers[index].relative_speed,
                // if the second of day is not found, use the speed modifier of the previous second of day
                Err(index) => {
                    if index > 0 {
                        modifier = speed_modifiers[index - 1].relative_speed;
                    } else {
                        modifier = speed_modifiers[0].relative_speed;
                    }
                }
            }
        }
        modifier
    }

    pub fn link_time_seconds_by_time_of_day(
        &self,
        link: &Link,
        second_of_day: SecondOfDay,
        day_of_week: DayOfWeek, 
    ) -> usize {
        if let Some(profile_id) = link.week_profile_ids[day_of_week] {
            let modifier = self.get_modifier_by_second_of_day(profile_id, second_of_day);
            (link.time_seconds() * (1.0/modifier)).round() as usize 

        } else {
            link.time_seconds().round() as usize
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_building_time_of_day_speeds() {
        let mut speeds_modifiers = HashMap::new();
        speeds_modifiers.insert(1, vec![(0, 1.0), (7200, 1.0), (3600, 0.5)]);
        let time_of_day_speeds = TimeOfDaySpeeds::new(speeds_modifiers);
        assert_eq!(time_of_day_speeds.get_modifier_by_second_of_day(1, 0), 1.0);
        assert_eq!(
            time_of_day_speeds.get_modifier_by_second_of_day(1, 3599),
            1.0
        );
        assert_eq!(
            time_of_day_speeds.get_modifier_by_second_of_day(1, 3600),
            0.5
        );
        assert_eq!(
            time_of_day_speeds.get_modifier_by_second_of_day(1, 7199),
            0.5
        );
        assert_eq!(
            time_of_day_speeds.get_modifier_by_second_of_day(1, 7200),
            1.0
        );
        assert_eq!(
            time_of_day_speeds.get_modifier_by_second_of_day(1, 10000),
            1.0
        );
    }
}
