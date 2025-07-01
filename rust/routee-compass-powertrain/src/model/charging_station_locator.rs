use std::{collections::HashMap, path::PathBuf};

use routee_compass_core::{
    model::{network::VertexId, traversal::TraversalModelError},
    util::fs::read_utils,
};
use serde::{Deserialize, Serialize};
use uom::si::f64::Power;

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ChargingStationConfig {
    vertex_id: VertexId,
    power_type: String,
    power_kw: f64,
    cost_per_kwh: f64,
}

#[derive(Debug)]
pub enum ChargingStation {
    L1 { power: Power, cost_per_kwh: f64 },
    L2 { power: Power, cost_per_kwh: f64 },
    DCFC { power: Power, cost_per_kwh: f64 },
}

impl ChargingStation {
    pub fn power(&self) -> Power {
        match self {
            ChargingStation::L1 { power, .. } => *power,
            ChargingStation::L2 { power, .. } => *power,
            ChargingStation::DCFC { power, .. } => *power,
        }
    }

    pub fn cost_per_kwh(&self) -> f64 {
        match self {
            ChargingStation::L1 { cost_per_kwh, .. } => *cost_per_kwh,
            ChargingStation::L2 { cost_per_kwh, .. } => *cost_per_kwh,
            ChargingStation::DCFC { cost_per_kwh, .. } => *cost_per_kwh,
        }
    }
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

    pub fn from_csv_file(file_path: &PathBuf) -> Result<Self, TraversalModelError> {
        let mut station_map = HashMap::new();

        let charging_stations = read_utils::from_csv::<ChargingStationConfig>(
            &file_path.as_path(),
            true,
            Some(kdam::Bar::builder().desc("charging stations")),
            None,
        )
        .map_err(|e| {
            TraversalModelError::BuildError(format!(
                "Failed to read charging stations from file {:?}: {}",
                file_path, e
            ))
        })?;
        for config in charging_stations {
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
