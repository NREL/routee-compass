use super::prediction::PredictionModelConfig;
use crate::model::{fieldname, prediction::PredictionModelRecord};
use routee_compass_core::model::{
    network::{Edge, Vertex},
    state::{InputFeature, StateFeature, StateModel, StateVariable},
    traversal::{TraversalModel, TraversalModelError, TraversalModelService},
    unit::{EnergyRateUnit, EnergyUnit},
};
use serde_json::Value;
use std::sync::Arc;
use uom::{si::f64::Energy, ConstZero};

#[derive(Clone)]
pub struct IceEnergyModel {
    pub prediction_model_record: Arc<PredictionModelRecord>,
}

impl IceEnergyModel {
    pub fn new(
        prediction_model_record: PredictionModelRecord,
    ) -> Result<Self, TraversalModelError> {
        Ok(Self {
            prediction_model_record: Arc::new(prediction_model_record),
        })
    }
}

impl TraversalModelService for IceEnergyModel {
    fn build(
        &self,
        _query: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModel>, TraversalModelError> {
        Ok(Arc::new(self.clone()))
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
        let ice_model = IceEnergyModel::new(prediction_model)?;
        Ok(ice_model)
    }
}

impl TraversalModel for IceEnergyModel {
    fn name(&self) -> String {
        format!("ICE Energy Model: {}", self.prediction_model_record.name)
    }
    fn input_features(&self) -> Vec<InputFeature> {
        let mut input_features = vec![InputFeature::Distance {
            name: String::from(fieldname::EDGE_DISTANCE),
            unit: None,
        }];
        input_features.extend(self.prediction_model_record.input_features.clone());
        input_features
    }

    fn output_features(&self) -> Vec<(String, StateFeature)> {
        vec![
            (
                String::from(fieldname::TRIP_ENERGY),
                StateFeature::Energy {
                    value: Energy::ZERO,
                    accumulator: true,
                    output_unit: Some(EnergyUnit::GallonsGasolineEquivalent),
                },
            ),
            (
                String::from(fieldname::EDGE_ENERGY),
                StateFeature::Energy {
                    value: Energy::ZERO,
                    accumulator: false,
                    output_unit: Some(EnergyUnit::GallonsGasolineEquivalent),
                },
            ),
        ]
    }

    fn traverse_edge(
        &self,
        _trajectory: (&Vertex, &Edge, &Vertex),
        state: &mut Vec<StateVariable>,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        ice_traversal(
            state,
            state_model,
            self.prediction_model_record.clone(),
            false,
        )
    }

    fn estimate_traversal(
        &self,
        _od: (&Vertex, &Vertex),
        state: &mut Vec<StateVariable>,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        ice_traversal(
            state,
            state_model,
            self.prediction_model_record.clone(),
            true,
        )
    }
}

fn ice_traversal(
    state: &mut [StateVariable],
    state_model: &StateModel,
    record: Arc<PredictionModelRecord>,
    estimate: bool,
) -> Result<(), TraversalModelError> {
    let distance = state_model.get_distance(state, fieldname::EDGE_DISTANCE)?;

    // generate energy for link traversal
    let energy = if estimate {
        match record.energy_rate_unit {
            EnergyRateUnit::GGPM => {
                let distance_miles = distance.get::<uom::si::length::mile>();
                let energy_gallons_gas = record.ideal_energy_rate * distance_miles;
                EnergyUnit::GallonsGasolineEquivalent.to_uom(energy_gallons_gas)
            }
            EnergyRateUnit::GDPM => {
                let distance_miles = distance.get::<uom::si::length::mile>();
                let energy_gallons_diesel = record.ideal_energy_rate * distance_miles;
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

    state_model.add_energy(state, fieldname::TRIP_ENERGY, &energy)?;
    state_model.set_energy(state, fieldname::EDGE_ENERGY, &energy)?;
    Ok(())
}
