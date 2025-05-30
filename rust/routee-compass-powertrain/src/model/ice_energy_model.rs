use super::prediction::PredictionModelConfig;
use crate::model::{fieldname, prediction::PredictionModelRecord};
use routee_compass_core::model::{
    network::{Edge, Vertex},
    state::{InputFeature, OutputFeature, StateModel, StateVariable},
    traversal::{TraversalModel, TraversalModelError, TraversalModelService},
    unit::Energy,
};
use serde_json::Value;
use std::sync::Arc;

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
                "failure reading prediction model configuration: {}",
                e
            ))
        })?;
        let prediction_model = PredictionModelRecord::try_from(&config)?;
        let ice_model = IceEnergyModel::new(prediction_model)?;
        Ok(ice_model)
    }
}

impl TraversalModel for IceEnergyModel {
    fn input_features(&self) -> Vec<(String, InputFeature)> {
        let mut input_features = vec![(
            String::from(fieldname::EDGE_DISTANCE),
            InputFeature::Distance(Some(self.prediction_model_record.distance_unit)),
        )];
        input_features.extend(self.prediction_model_record.input_features.clone());
        input_features
    }

    fn output_features(&self) -> Vec<(String, OutputFeature)> {
        let energy_unit = self
            .prediction_model_record
            .energy_rate_unit
            .associated_energy_unit();
        vec![
            (
                String::from(fieldname::TRIP_ENERGY),
                OutputFeature::Energy {
                    energy_unit,
                    initial: Energy::ZERO,
                    accumulator: true,
                },
            ),
            (
                String::from(fieldname::EDGE_ENERGY),
                OutputFeature::Energy {
                    energy_unit,
                    initial: Energy::ZERO,
                    accumulator: false,
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
    let (distance, distance_unit) =
        state_model.get_distance(state, fieldname::EDGE_DISTANCE, None)?;

    // generate energy for link traversal
    let (energy, energy_unit) = if estimate {
        Energy::create(
            (&distance, distance_unit),
            (&record.ideal_energy_rate, &record.energy_rate_unit),
        )?
    } else {
        record.predict(state, state_model)?
    };

    state_model.add_energy(state, fieldname::TRIP_ENERGY, &energy, &energy_unit)?;
    state_model.set_energy(state, fieldname::EDGE_ENERGY, &energy, &energy_unit)?;
    Ok(())
}
