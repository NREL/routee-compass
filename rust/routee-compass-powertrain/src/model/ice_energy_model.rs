use super::prediction::PredictionModelConfig;
use crate::model::{fieldname, prediction::PredictionModelRecord};
use routee_compass_core::{
    algorithm::search::SearchTree,
    model::{
        network::{Edge, Vertex},
        state::{InputFeature, StateModel, StateVariable, StateVariableConfig},
        traversal::{TraversalModel, TraversalModelError, TraversalModelService},
        unit::{EnergyRateUnit, EnergyUnit},
    },
};
use serde_json::Value;
use std::sync::Arc;
use uom::{si::f64::Energy, ConstZero};

#[derive(Clone)]
pub struct IceEnergyModel {
    pub prediction_model_record: Arc<PredictionModelRecord>,
    pub include_trip_energy: bool,
    // Pre-resolved indices for performance
    edge_distance_idx: usize,
    edge_energy_idx: usize,
    trip_energy_idx: Option<usize>,
}

impl IceEnergyModel {
    pub fn new(
        prediction_model_record: Arc<PredictionModelRecord>,
        include_trip_energy: bool,
        edge_distance_idx: usize,
        edge_energy_idx: usize,
        trip_energy_idx: Option<usize>,
    ) -> Result<Self, TraversalModelError> {
        Ok(Self {
            prediction_model_record,
            include_trip_energy,
            edge_distance_idx,
            edge_energy_idx,
            trip_energy_idx,
        })
    }
}

impl TraversalModelService for IceEnergyModel {
    fn input_features(&self) -> Vec<InputFeature> {
        let mut input_features = vec![InputFeature::Distance {
            name: String::from(fieldname::EDGE_DISTANCE),
            unit: None,
        }];
        input_features.extend(self.prediction_model_record.input_features.clone());
        input_features
    }

    fn output_features(&self) -> Vec<(String, StateVariableConfig)> {
        let mut features = vec![(
            String::from(fieldname::EDGE_ENERGY_LIQUID),
            StateVariableConfig::Energy {
                initial: Energy::ZERO,
                accumulator: false,
                output_unit: Some(
                    self.prediction_model_record
                        .energy_rate_unit
                        .associated_energy_unit(),
                ),
            },
        )];
        if self.include_trip_energy {
            features.push((
                String::from(fieldname::TRIP_ENERGY_LIQUID),
                StateVariableConfig::Energy {
                    initial: Energy::ZERO,
                    accumulator: true,
                    output_unit: Some(
                        self.prediction_model_record
                            .energy_rate_unit
                            .associated_energy_unit(),
                    ),
                },
            ));
        }
        features
    }

    fn build(
        &self,
        _query: &serde_json::Value,
        state_model: Arc<StateModel>,
    ) -> Result<Arc<dyn TraversalModel>, TraversalModelError> {
        let edge_distance_idx = state_model
            .get_index(fieldname::EDGE_DISTANCE)
            .map_err(|e| {
                TraversalModelError::BuildError(format!(
                    "Failed to find EDGE_DISTANCE index: {}",
                    e
                ))
            })?;

        let edge_energy_idx = state_model
            .get_index(fieldname::EDGE_ENERGY_LIQUID)
            .map_err(|e| {
                TraversalModelError::BuildError(format!(
                    "Failed to find EDGE_ENERGY_LIQUID index: {}",
                    e
                ))
            })?;

        let trip_energy_idx = if self.include_trip_energy {
            Some(
                state_model
                    .get_index(fieldname::TRIP_ENERGY_LIQUID)
                    .map_err(|e| {
                        TraversalModelError::BuildError(format!(
                            "Failed to find TRIP_ENERGY_LIQUID index: {}",
                            e
                        ))
                    })?,
            )
        } else {
            None
        };

        let model = IceEnergyModel::new(
            self.prediction_model_record.clone(),
            self.include_trip_energy,
            edge_distance_idx,
            edge_energy_idx,
            trip_energy_idx,
        )?;
        Ok(Arc::new(model))
    }
}

impl TryFrom<&Value> for IceEnergyModel {
    type Error = TraversalModelError;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let config: PredictionModelConfig = serde_json::from_value(value.clone()).map_err(|e| {
            TraversalModelError::BuildError(format!(
                "failure reading prediction model configuration: {e}"
            ))
        })?;
        let prediction_model = PredictionModelRecord::try_from(&config)?;
        let include_trip_energy = match value.get("include_trip_energy") {
            Some(v) => {
                v.as_bool().ok_or_else(|| {
                    TraversalModelError::BuildError("Failed to parse the parameter `include_trip_energy` as a boolean when building the ICE Energy model".to_string())
                })?
            },
            None => true
        };
        // Note: indices are dummy values and will be set properly in build()
        let ice_model = IceEnergyModel::new(
            Arc::new(prediction_model),
            include_trip_energy,
            0,    // dummy edge_distance_idx
            0,    // dummy edge_energy_idx
            None, // dummy trip_energy_idx
        )?;
        Ok(ice_model)
    }
}

impl TraversalModel for IceEnergyModel {
    fn name(&self) -> String {
        format!("ICE Energy Model: {}", self.prediction_model_record.name)
    }

    fn traverse_edge(
        &self,
        _trajectory: (&Vertex, &Edge, &Vertex),
        state: &mut Vec<StateVariable>,
        _tree: &SearchTree,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        ice_traversal(
            state,
            state_model,
            self.prediction_model_record.clone(),
            self.edge_distance_idx,
            self.edge_energy_idx,
            self.trip_energy_idx,
            false,
        )
    }

    fn estimate_traversal(
        &self,
        _od: (&Vertex, &Vertex),
        state: &mut Vec<StateVariable>,
        _tree: &SearchTree,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        ice_traversal(
            state,
            state_model,
            self.prediction_model_record.clone(),
            self.edge_distance_idx,
            self.edge_energy_idx,
            self.trip_energy_idx,
            true,
        )
    }
}

fn ice_traversal(
    state: &mut [StateVariable],
    state_model: &StateModel,
    record: Arc<PredictionModelRecord>,
    edge_distance_idx: usize,
    edge_energy_idx: usize,
    trip_energy_idx: Option<usize>,
    estimate: bool,
) -> Result<(), TraversalModelError> {
    let distance = state_model.get_distance_by_index(state, edge_distance_idx)?;

    // generate energy for link traversal
    let energy = if estimate {
        match record.energy_rate_unit {
            EnergyRateUnit::GGPM => {
                let distance_miles = distance.get::<uom::si::length::mile>();
                let energy_gallons_gas = record.a_star_heuristic_energy_rate * distance_miles;
                EnergyUnit::GallonsGasolineEquivalent.to_uom(energy_gallons_gas)
            }
            EnergyRateUnit::GDPM => {
                let distance_miles = distance.get::<uom::si::length::mile>();
                let energy_gallons_diesel = record.a_star_heuristic_energy_rate * distance_miles;
                EnergyUnit::GallonsDieselEquivalent.to_uom(energy_gallons_diesel)
            }
            _ => {
                return Err(TraversalModelError::InternalError(format!(
                    "Unsupported energy rate unit: {}",
                    record.energy_rate_unit
                )));
            }
        }
    } else {
        record.predict(state, state_model)?
    };

    if let Some(idx) = trip_energy_idx {
        state_model.add_energy_by_index(state, idx, &energy)?;
    }
    state_model.set_energy_by_index(state, edge_energy_idx, &energy)?;
    Ok(())
}
