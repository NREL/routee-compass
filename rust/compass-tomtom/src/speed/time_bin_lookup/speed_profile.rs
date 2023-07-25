use super::{
    speed_lookup_error::SpeedLookupError, speed_profile_id::SpeedProfileId, weekday::Weekday,
    weekday_speed_profile_id::WeekdaySpeedProfileId,
};
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use std::{collections::LinkedList, fs::File, io::BufReader};

#[derive(Serialize, Deserialize)]
// #[serde(rename_all = "PascalCase")]
pub struct SpeedProfile {
    speed_profile_id: SpeedProfileId,
    monday_profile_id: WeekdaySpeedProfileId,
    tuesday_profile_id: WeekdaySpeedProfileId,
    wednesday_profile_id: WeekdaySpeedProfileId,
    thursday_profile_id: WeekdaySpeedProfileId,
    friday_profile_id: WeekdaySpeedProfileId,
    saturday_profile_id: WeekdaySpeedProfileId,
    sunday_profile_id: WeekdaySpeedProfileId,
}

impl SpeedProfile {
    pub fn get_id(&self, weekday: &Weekday) -> WeekdaySpeedProfileId {
        match weekday {
            Weekday::Monday => self.monday_profile_id,
            Weekday::Tuesday => self.tuesday_profile_id,
            Weekday::Wednesday => self.wednesday_profile_id,
            Weekday::Thursday => self.thursday_profile_id,
            Weekday::Friday => self.friday_profile_id,
            Weekday::Saturday => self.saturday_profile_id,
            Weekday::Sunday => self.sunday_profile_id,
        }
    }
}

/// builds a lookup table from speed profile index to weekday speed profile id
fn build_speed_profile_lookup(
    file: File,
    weekday: Weekday,
    is_gzip: bool,
) -> Result<Vec<WeekdaySpeedProfileId>, SpeedLookupError> {
    // read in CSV SpeedProfile rows, collecting the WeekdaySpeedProfileIds for the given weekday
    // let mut reader = csv::Reader::from_reader(file);
    let mut reader: csv::Reader<Box<dyn std::io::Read>> = if is_gzip {
        csv::Reader::from_reader(Box::new(BufReader::new(GzDecoder::new(file))))
    } else {
        csv::Reader::from_reader(Box::new(file))
    };

    let mut buffer: LinkedList<(SpeedProfileId, WeekdaySpeedProfileId)> = LinkedList::new();
    for row in reader.deserialize() {
        let speed_profile: SpeedProfile =
            row.map_err(|e| SpeedLookupError::DeserializationError {
                msg: format!("failure reading speed profile row: {}", e.to_string()),
            })?;
        let speed_profile_id = speed_profile.speed_profile_id;
        let weekday_speed_profile_id = speed_profile.get_id(&weekday);
        buffer.push_back((speed_profile_id, weekday_speed_profile_id));
    }
    // build the result lookup table (once we know the input size)
    let mut result: Vec<WeekdaySpeedProfileId> = vec![WeekdaySpeedProfileId::UNSET; buffer.len()];
    for (speed_profile_id, weekday_speed_profile_id) in buffer {
        let index = speed_profile_id.0 as usize;
        match result.get(index) {
            None => {
                result.insert(index, weekday_speed_profile_id);
                Ok(())
            }
            Some(_) => {
                let error = SpeedLookupError::NonUniqueSpeedProfileId { speed_profile_id };
                Err(error)
            }
        }?
    }
    Ok(result)
}
