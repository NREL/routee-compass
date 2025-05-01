use crate::model::prediction::PredictionModelRecord;
use routee_compass_core::model::{
    network::{Edge, Vertex},
    state::{InputFeature, OutputFeature, StateModel, StateVariable},
    traversal::{TraversalModel, TraversalModelError},
    unit::Energy,
};
use std::sync::Arc;

pub struct IceEnergyModel {
    pub prediction_model_record: Arc<PredictionModelRecord>,
}

impl IceEnergyModel {
    const EDGE_ENERGY_LIQUID: &'static str = "edge_energy_liquid";
    const TRIP_ENERGY_LIQUID: &'static str = "trip_energy_liquid";
    const EDGE_DISTANCE: &'static str = "edge_distance";
    const EDGE_SPEED: &'static str = "edge_speed";
    const EDGE_GRADE: &'static str = "edge_grade";

    pub fn new(
        prediction_model_record: PredictionModelRecord,
    ) -> Result<Self, TraversalModelError> {
        Ok(Self {
            prediction_model_record: Arc::new(prediction_model_record),
        })
    }
}

impl TraversalModel for IceEnergyModel {
    fn input_features(&self) -> Vec<(String, InputFeature)> {
        vec![
            (
                String::from(IceEnergyModel::EDGE_DISTANCE),
                InputFeature::Distance(Some(
                    self.prediction_model_record
                        .speed_unit
                        .associated_distance_unit(),
                )),
            ),
            (
                String::from(IceEnergyModel::EDGE_SPEED),
                InputFeature::Speed(Some(self.prediction_model_record.speed_unit)),
            ),
            (
                String::from(IceEnergyModel::EDGE_GRADE),
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
                String::from(IceEnergyModel::TRIP_ENERGY_LIQUID),
                OutputFeature::Energy {
                    energy_unit,
                    initial: Energy::ZERO,
                },
            ),
            (
                String::from(IceEnergyModel::EDGE_ENERGY_LIQUID),
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
    let distance_unit = prediction_model_record
        .speed_unit
        .associated_distance_unit();
    let speed_unit = prediction_model_record.speed_unit;
    let grade_unit = prediction_model_record.grade_unit;

    let (distance, _) =
        state_model.get_distance(state, IceEnergyModel::EDGE_DISTANCE, Some(&distance_unit))?;
    let (speed, _) = state_model.get_speed(state, IceEnergyModel::EDGE_SPEED, Some(&speed_unit))?;
    let (grade, _) = state_model.get_grade(state, IceEnergyModel::EDGE_GRADE, Some(&grade_unit))?;

    let (energy, _energy_unit) = prediction_model_record.predict(
        (speed, speed_unit),
        (grade, grade_unit),
        (distance, distance_unit),
    )?;

    state_model.add_energy(
        state,
        IceEnergyModel::TRIP_ENERGY_LIQUID,
        &energy,
        &prediction_model_record
            .energy_rate_unit
            .associated_energy_unit(),
    )?;
    state_model.set_energy(
        state,
        IceEnergyModel::EDGE_ENERGY_LIQUID,
        &energy,
        &prediction_model_record
            .energy_rate_unit
            .associated_energy_unit(),
    )?;
    Ok(())
}
