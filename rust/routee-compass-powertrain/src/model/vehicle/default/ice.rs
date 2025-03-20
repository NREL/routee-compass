use crate::model::{prediction::PredictionModelRecord, vehicle::VehicleType};
use routee_compass_core::model::{
    state::{StateFeature, StateModel, StateVariable},
    traversal::TraversalModelError,
    unit::{Distance, DistanceUnit, Energy, EnergyUnit, Grade, GradeUnit, Speed, SpeedUnit},
};
use std::sync::Arc;

pub struct ICE {
    pub name: String,
    pub prediction_model_record: Arc<PredictionModelRecord>,
}

impl ICE {
    const ENERGY_FEATURE_NAME: &'static str = "energy_liquid";
    pub fn new(
        name: String,
        prediction_model_record: PredictionModelRecord,
    ) -> Result<Self, TraversalModelError> {
        Ok(Self {
            name,
            prediction_model_record: Arc::new(prediction_model_record),
        })
    }
}

impl VehicleType for ICE {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn state_features(&self) -> Vec<(String, StateFeature)> {
        let energy_unit = self
            .prediction_model_record
            .energy_rate_unit
            .associated_energy_unit();
        vec![(
            String::from(ICE::ENERGY_FEATURE_NAME),
            StateFeature::Energy {
                energy_unit,
                initial: Energy::ZERO,
            },
        )]
    }
    fn best_case_energy(
        &self,
        distance: (Distance, DistanceUnit),
    ) -> Result<(Energy, EnergyUnit), TraversalModelError> {
        let (distance, distance_unit) = distance;
        let energy = Energy::create(
            (&distance, &distance_unit),
            (
                &self.prediction_model_record.ideal_energy_rate,
                &self.prediction_model_record.energy_rate_unit,
            ),
        )?;
        Ok(energy)
    }

    fn best_case_energy_state(
        &self,
        distance: (Distance, DistanceUnit),
        state: &mut Vec<StateVariable>,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        let (energy, _energy_unit) = self.best_case_energy(distance)?;
        state_model.add_energy(
            state,
            &ICE::ENERGY_FEATURE_NAME.into(),
            &energy,
            &self
                .prediction_model_record
                .energy_rate_unit
                .associated_energy_unit(),
        )?;
        Ok(())
    }

    fn consume_energy(
        &self,
        speed: (Speed, SpeedUnit),
        grade: (Grade, GradeUnit),
        distance: (Distance, DistanceUnit),
        state: &mut Vec<StateVariable>,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        let (energy, _energy_unit) = self
            .prediction_model_record
            .predict(speed, grade, distance)?;
        state_model.add_energy(
            state,
            &ICE::ENERGY_FEATURE_NAME.into(),
            &energy,
            &self
                .prediction_model_record
                .energy_rate_unit
                .associated_energy_unit(),
        )?;
        Ok(())
    }

    fn update_from_query(
        &self,
        _query: &serde_json::Value,
    ) -> Result<Arc<dyn VehicleType>, TraversalModelError> {
        // just return a clone of self
        Ok(Arc::new(ICE {
            name: self.name.clone(),
            prediction_model_record: self.prediction_model_record.clone(),
        }))
    }
}
