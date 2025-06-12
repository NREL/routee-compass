use routee_compass_core::model::{
    traversal::TraversalModelError,
    unit::{EnergyRateUnit},
};

pub trait PredictionModel: Send + Sync {
    fn predict(
        &self,
        feature_vector: &Vec<f64>,
    ) -> Result<(f64, EnergyRateUnit), TraversalModelError>;
}
