use routee_compass_core::{
    model::traversal::traversal_model_error::TraversalModelError,
    util::unit::{EnergyRate, EnergyRateUnit, Grade, GradeUnit, Speed, SpeedUnit},
};

pub trait SpeedGradePredictionModel: Send + Sync {
    fn predict(
        &self,
        speed: Speed,
        speed_unit: SpeedUnit,
        grade: Grade,
        grade_unit: GradeUnit,
    ) -> Result<(EnergyRate, EnergyRateUnit), TraversalModelError>;
}
