use routee_compass_core::{
    model::traversal::traversal_model_error::TraversalModelError,
    util::unit::{EnergyRate, EnergyRateUnit, Grade, GradeUnit, Speed, SpeedUnit},
};

pub trait PredictionModel: Send + Sync {
    fn predict(
        &self,
        speed: (Speed, SpeedUnit),
        grade: (Grade, GradeUnit),
    ) -> Result<(EnergyRate, EnergyRateUnit), TraversalModelError>;
}
