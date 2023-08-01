use std::{collections::LinkedList, fs::File, io::BufReader};

use flate2::read::GzDecoder;
use serde::Deserialize;

use super::{
    network_id::NetworkId, speed_lookup_error::SpeedLookupError, speed_profile_id::SpeedProfileId,
};

#[derive(Debug, Deserialize)]
pub struct Netw2SpeedProfile {
    network_id: NetworkId,
    speed_profile_id: SpeedProfileId,
}

/// builds a lookup table from speed profile index to weekday speed profile id
pub fn build_speed_profile_lookup(
    file: File,
    is_gzip: bool,
) -> Result<Vec<SpeedProfileId>, SpeedLookupError> {
    // from GZIP: https://stackoverflow.com/a/65778271
    // generic reader for GZIP or regular CSV: https://stackoverflow.com/a/64564120
    let mut reader: csv::Reader<Box<dyn std::io::Read>> = if is_gzip {
        csv::Reader::from_reader(Box::new(BufReader::new(GzDecoder::new(file))))
    } else {
        csv::Reader::from_reader(Box::new(file))
    };

    // read in the CSV records
    let mut buffer: LinkedList<Netw2SpeedProfile> = LinkedList::new();
    for row in reader.deserialize() {
        let nw2sp: Netw2SpeedProfile = row.map_err(|e| SpeedLookupError::DeserializationError {
            msg: format!("failure reading speed profile row: {}", e.to_string()),
        })?;
        buffer.push_back(nw2sp);
    }

    // build the result lookup table (once we know the input size)
    let mut result: Vec<SpeedProfileId> = vec![SpeedProfileId::UNSET; buffer.len()];
    for row in buffer {
        let index = row.network_id.0 as usize;
        match result.get(index) {
            None => {
                result.insert(index, row.speed_profile_id);
                Ok(())
            }
            Some(_) => {
                let error = SpeedLookupError::NonUniqueNetworkId {
                    network_id: row.network_id,
                };
                Err(error)
            }
        }?
    }
    Ok(result)
}
