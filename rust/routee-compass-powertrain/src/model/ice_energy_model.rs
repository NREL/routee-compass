use crate::model::prediction::PredictionModelRecord;
use routee_compass_core::model::{
    network::{Edge, Vertex},
    state::{StateFeature, StateModel, StateVariable},
    traversal::{TraversalModel, TraversalModelError},
    unit::{Distance, Energy, Grade, Speed},
};
use std::sync::Arc;

pub struct IceEnergyModel {
    pub prediction_model_record: Arc<PredictionModelRecord>,
}

impl IceEnergyModel {
    const LEG_ENERGY_LIQUID: &'static str = "leg_energy_liquid";
    const TRIP_ENERGY_LIQUID: &'static str = "trip_energy_liquid";
    const LEG_DISTANCE: &'static str = "leg_distance";
    const LEG_SPEED: &'static str = "leg_speed";
    const LEG_GRADE: &'static str = "leg_grade";

    pub fn new(
        prediction_model_record: PredictionModelRecord,
    ) -> Result<Self, TraversalModelError> {
        Ok(Self {
            prediction_model_record: Arc::new(prediction_model_record),
        })
    }
}

impl TraversalModel for IceEnergyModel {
    fn input_features(&self) -> Vec<(String, StateFeature)> {
        vec![
            (
                String::from(IceEnergyModel::LEG_DISTANCE),
                StateFeature::Distance {
                    distance_unit: self
                        .prediction_model_record
                        .speed_unit
                        .associated_distance_unit(),
                    initial: Distance::ZERO,
                },
            ),
            (
                String::from(IceEnergyModel::LEG_SPEED),
                StateFeature::Speed {
                    speed_unit: self.prediction_model_record.speed_unit,
                    initial: Speed::ZERO,
                },
            ),
            (
                String::from(IceEnergyModel::LEG_GRADE),
                StateFeature::Grade {
                    grade_unit: self.prediction_model_record.grade_unit,
                    initial: Grade::ZERO,
                },
            ),
        ]
    }

    fn output_features(&self) -> Vec<(String, StateFeature)> {
        let energy_unit = self
            .prediction_model_record
            .energy_rate_unit
            .associated_energy_unit();
        vec![
            (
                String::from(IceEnergyModel::TRIP_ENERGY_LIQUID),
                StateFeature::Energy {
                    energy_unit,
                    initial: Energy::ZERO,
                },
            ),
            (
                String::from(IceEnergyModel::LEG_ENERGY_LIQUID),
                StateFeature::Energy {
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
    let distance = state_model.get_distance(state, IceEnergyModel::LEG_DISTANCE, &distance_unit)?;
    let speed = state_model.get_speed(state, IceEnergyModel::LEG_SPEED, &speed_unit)?;
    let grade = state_model.get_grade(state, IceEnergyModel::LEG_GRADE, &grade_unit)?;
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
        IceEnergyModel::LEG_ENERGY_LIQUID,
        &energy,
        &prediction_model_record
            .energy_rate_unit
            .associated_energy_unit(),
    )?;
    Ok(())
}
