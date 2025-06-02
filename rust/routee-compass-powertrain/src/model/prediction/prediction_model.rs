use routee_compass_core::model::{
    traversal::TraversalModelError,
    unit::{EnergyRate, EnergyRateUnit},
};

pub trait PredictionModel: Send + Sync {
    fn predict(
        &self,
        feature_vector: &Vec<f64>,
    ) -> Result<(EnergyRate, EnergyRateUnit), TraversalModelError>;
}
