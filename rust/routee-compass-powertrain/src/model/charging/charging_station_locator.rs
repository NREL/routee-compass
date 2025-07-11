use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    str::FromStr,
};

use kdam::Bar;
use routee_compass_core::{
    model::{
        map::{DistanceTolerance, NearestSearchResult, SpatialIndex},
        network::{Vertex, VertexId},
        traversal::TraversalModelError,
    },
    util::fs::read_utils,
};
use serde::{Deserialize, Serialize};
use uom::si::f64::Power;

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ChargingStationConfig {
    power_type: String,
    power_kw: f64,
    cost_per_kwh: f64,
    x: f32,
    y: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum PowerType {
    L1,
    L2,
    DCFC,
}

impl PowerType {
    pub fn all() -> Vec<PowerType> {
        vec![PowerType::L1, PowerType::L2, PowerType::DCFC]
    }
}

impl FromStr for PowerType {
    type Err = TraversalModelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "l1" => Ok(PowerType::L1),
            "l2" => Ok(PowerType::L2),
            "dcfc" => Ok(PowerType::DCFC),
            _ => Err(TraversalModelError::BuildError(format!(
                "Unknown power type: {}",
                s
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ChargingStation {
    pub power_type: PowerType,
    pub power: Power,
    pub cost_per_kwh: f64,
}

impl TryFrom<ChargingStationConfig> for ChargingStation {
    type Error = TraversalModelError;
    fn try_from(config: ChargingStationConfig) -> Result<Self, TraversalModelError> {
        let power = Power::new::<uom::si::power::kilowatt>(config.power_kw);
        match config.power_type.to_lowercase().as_str() {
            "l1" => Ok(ChargingStation {
                power_type: PowerType::L1,
                power,
                cost_per_kwh: config.cost_per_kwh,
            }),
            "l2" => Ok(ChargingStation {
                power_type: PowerType::L2,
                power,
                cost_per_kwh: config.cost_per_kwh,
            }),
            "dcfc" => Ok(ChargingStation {
                power_type: PowerType::DCFC,
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

    pub fn from_csv_files(
        charge_station_filepath: &PathBuf,
        vertex_filepath: &PathBuf,
        station_match_tolerance: Option<DistanceTolerance>,
    ) -> Result<Self, TraversalModelError> {
        let vertices: Box<[Vertex]> = read_utils::from_csv(
            &vertex_filepath,
            true,
            Some(Bar::builder().desc("graph vertices")),
            None,
        )
        .map_err(|e| {
            TraversalModelError::BuildError(format!(
                "Failed to read vertices from file {:?}: {}",
                vertex_filepath, e
            ))
        })?;

        let spatial_index = SpatialIndex::new_vertex_oriented(&vertices, station_match_tolerance);

        let mut station_map = HashMap::new();

        let charging_stations = read_utils::from_csv::<ChargingStationConfig>(
            &charge_station_filepath.as_path(),
            true,
            Some(kdam::Bar::builder().desc("charging stations")),
            None,
        )
        .map_err(|e| {
            TraversalModelError::BuildError(format!(
                "Failed to read charging stations from file {:?}: {}",
                charge_station_filepath, e
            ))
        })?;
        for config in charging_stations {
            let point = geo::Point::new(config.x, config.y);
            let nearest = spatial_index.nearest_graph_id(&point).map_err(|e| {
                TraversalModelError::BuildError(format!(
                    "Failed to find nearest vertex for charging station at ({}, {}): {}",
                    config.x, config.y, e
                ))
            })?;
            let vertex_id = match nearest {
                NearestSearchResult::NearestVertex(v_id) => v_id,
                _ => {
                    return Err(TraversalModelError::BuildError(format!(
                        "Expected nearest vertex, found edge for charging station at ({}, {})",
                        config.x, config.y
                    )))
                }
            };
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

    pub fn with_power_types(&self, power_types: &HashSet<PowerType>) -> Self {
        let filtered_stations: HashMap<VertexId, ChargingStation> = self
            .station_map
            .clone()
            .into_iter()
            .filter_map(|(vertex_id, station)| {
                if power_types.contains(&station.power_type) {
                    Some((vertex_id, station))
                } else {
                    None
                }
            })
            .collect();
        ChargingStationLocator {
            station_map: filtered_stations,
        }
    }
}
