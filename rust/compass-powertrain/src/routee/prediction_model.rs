use compass_core::{
    model::traversal::traversal_model_error::TraversalModelError,
    util::unit::{EnergyRate, EnergyRateUnit, Speed, SpeedUnit, GradeUnit, Grade},
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
