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
        vec![
            (
                String::from(fieldname::EDGE_DISTANCE),
                InputFeature::Distance(Some(
                    self.prediction_model_record
                        .speed_unit
                        .associated_distance_unit(),
                )),
            ),
            (
                String::from(fieldname::EDGE_SPEED),
                InputFeature::Speed(Some(self.prediction_model_record.speed_unit)),
            ),
            (
                String::from(fieldname::EDGE_GRADE),
                InputFeature::Grade(Some(self.prediction_model_record.grade_unit)),
            ),
        ]
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
                },
            ),
            (
                String::from(fieldname::EDGE_ENERGY),
                OutputFeature::Energy {
                    energy_unit,
                    initial: Energy::ZERO,
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
        ice_traversal(state, state_model, self.prediction_model_record.clone())
    }

    fn estimate_traversal(
        &self,
        _od: (&Vertex, &Vertex),
        state: &mut Vec<StateVariable>,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        ice_traversal(state, state_model, self.prediction_model_record.clone())
    }
}

fn ice_traversal(
    state: &mut [StateVariable],
    state_model: &StateModel,
    prediction_model_record: Arc<PredictionModelRecord>,
) -> Result<(), TraversalModelError> {
    let (distance, distance_unit) =
        state_model.get_distance(state, fieldname::EDGE_DISTANCE, None)?;
    let (speed, speed_unit) = state_model.get_speed(state, fieldname::EDGE_SPEED, None)?;
    let (grade, grade_unit) = state_model.get_grade(state, fieldname::EDGE_GRADE, None)?;

    let (energy, energy_unit) = prediction_model_record.predict(
        (speed, speed_unit),
        (grade, grade_unit),
        (distance, distance_unit),
    )?;

    state_model.add_energy(state, fieldname::TRIP_ENERGY, &energy, &energy_unit)?;
    state_model.set_energy(state, fieldname::EDGE_ENERGY, &energy, &energy_unit)?;
    Ok(())
}
