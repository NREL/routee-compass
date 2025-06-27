use std::collections::HashMap;

use routee_compass_core::model::{network::VertexId, traversal::TraversalModelError};
use serde::{Deserialize, Serialize};
use uom::si::f64::Power;

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ChargingStationConfig {
    vertex_id: VertexId,
    power_type: String,
    power_kw: f64,
    cost_per_kwh: f64,
}

pub enum ChargingStation {
    L1 { power: Power, cost_per_kwh: f64 },
    L2 { power: Power, cost_per_kwh: f64 },
    DCFC { power: Power, cost_per_kwh: f64 },
}

impl TryFrom<ChargingStationConfig> for ChargingStation {
    type Error = TraversalModelError;
    fn try_from(config: ChargingStationConfig) -> Result<Self, TraversalModelError> {
        let power = Power::new::<uom::si::power::kilowatt>(config.power_kw);
        match config.power_type.to_lowercase().as_str() {
            "l1" => Ok(ChargingStation::L1 {
                power,
                cost_per_kwh: config.cost_per_kwh,
            }),
            "l2" => Ok(ChargingStation::L2 {
                power,
                cost_per_kwh: config.cost_per_kwh,
            }),
            "dcfc" => Ok(ChargingStation::DCFC {
                power,
                cost_per_kwh: config.cost_per_kwh,
            }),
            _ => Err(TraversalModelError::BuildError(format!(
                "Unknown charging station type: {}",
                config.power_type
            ))),
        }
    }
}

#[derive(Default)]
pub struct ChargingStationLocator {
    station_map: HashMap<VertexId, ChargingStation>,
}

impl ChargingStationLocator {
    pub fn new(station_map: HashMap<VertexId, ChargingStation>) -> Self {
        Self { station_map }
    }

    pub fn from_csv_file(file_path: &str) -> Result<Self, TraversalModelError> {
        let mut station_map = HashMap::new();
        // csv file is expected to have lines in the format:
        // vertex_id,power_type,power_kw,cost_per_kwh
        let mut rdr = csv::Reader::from_path(file_path).map_err(|e| {
            TraversalModelError::BuildError(format!(
                "Failed to read charging station CSV file {}: {}",
                file_path, e
            ))
        })?;
        for result in rdr.deserialize() {
            let config: ChargingStationConfig = result.map_err(|e| {
                TraversalModelError::BuildError(format!(
                    "Failed to deserialize charging station config: {}",
                    e
                ))
            })?;
            let vertex_id = config.vertex_id;
            let station: ChargingStation = config.try_into()?;
            station_map.insert(vertex_id, station);
        }
        Ok(ChargingStationLocator::new(station_map))
    }

    pub fn add_station(&mut self, vertex_id: VertexId, station: ChargingStation) {
        self.station_map.insert(vertex_id, station);
    }

    pub fn get_station(&self, vertex_id: &VertexId) -> Option<&ChargingStation> {
        self.station_map.get(vertex_id)
    }
}
