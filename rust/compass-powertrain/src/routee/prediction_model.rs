use compass_core::{
    model::traversal::traversal_model_error::TraversalModelError,
    util::unit::{EnergyRate, EnergyRateUnit, Speed, SpeedUnit},
};

pub trait SpeedGradePredictionModel: Send + Sync {
    fn predict(
        &self,
        speed: Speed,
        speed_unit: SpeedUnit,
        grade: f64,
    ) -> Result<(EnergyRate, EnergyRateUnit), TraversalModelError>;
}
