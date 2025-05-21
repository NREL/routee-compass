use routee_compass_core::model::{
    state::{InputFeature, StateModel, StateVariable},
    traversal::TraversalModelError,
    unit::{EnergyRate, EnergyRateUnit},
};

pub trait PredictionModel: Send + Sync {
    fn predict(
        &self,
        input_features: &[(String, InputFeature)],
        state: &mut Vec<StateVariable>,
        state_model: &StateModel,
    ) -> Result<(EnergyRate, EnergyRateUnit), TraversalModelError>;
}
