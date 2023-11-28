use std::path::Path;

use crate::routee::prediction::prediction_model::PredictionModel;
use ndarray::CowArray;
use ort::{
    tensor::OrtOwnedTensor, Environment, GraphOptimizationLevel, Session, SessionBuilder, Value,
};
use routee_compass_core::{
    model::traversal::traversal_model_error::TraversalModelError,
    util::unit::{as_f64::AsF64, EnergyRate, EnergyRateUnit, Grade, GradeUnit, Speed, SpeedUnit},
};

pub struct OnnxSpeedGradeModel {
    session: Session,
    speed_unit: SpeedUnit,
    grade_unit: GradeUnit,
    energy_rate_unit: EnergyRateUnit,
}

impl PredictionModel for OnnxSpeedGradeModel {
    fn predict(
        &self,
        speed: (Speed, SpeedUnit),
        grade: (Grade, GradeUnit),
    ) -> Result<(EnergyRate, EnergyRateUnit), TraversalModelError> {
        let (speed, speed_unit) = speed;
        let (grade, grade_unit) = grade;
        let speed_value: f32 = speed_unit.convert(speed, self.speed_unit).as_f64() as f32;
        let grade_value: f32 = grade_unit.convert(grade, self.grade_unit).as_f64() as f32;
        let array = ndarray::Array1::from(vec![speed_value, grade_value])
            .into_shape((1, 2))
            .map_err(|e| {
                TraversalModelError::PredictionModel(format!(
                    "Failed to reshape input for prediction: {}",
                    e
                ))
            })?;

        let x = CowArray::from(array).into_dyn();
        let value = Value::from_array(self.session.allocator(), &x).map_err(|e| {
            TraversalModelError::PredictionModel(format!(
                "Failed to create input value for prediction: {}",
                e
            ))
        })?;
        let input = vec![value];

        let result: OrtOwnedTensor<f32, _> =
            self.session.run(input).unwrap()[0].try_extract().unwrap();
        let output_f64 = result.view().to_owned().into_raw_vec()[0] as f64;

        let energy_rate = EnergyRate::new(output_f64);
        Ok((energy_rate, self.energy_rate_unit))
    }
}

impl OnnxSpeedGradeModel {
    pub fn new<P: AsRef<Path>>(
        onnx_model_path: &P,
        speed_unit: SpeedUnit,
        grade_unit: GradeUnit,
        energy_rate_unit: EnergyRateUnit,
    ) -> Result<Self, TraversalModelError> {
        let env = Environment::builder()
            .build()
            .map_err(|e| TraversalModelError::BuildError(e.to_string()))?
            .into_arc();

        let session = SessionBuilder::new(&env)
            .map_err(|e| TraversalModelError::BuildError(e.to_string()))?
            .with_intra_threads(1)
            .map_err(|e| TraversalModelError::BuildError(e.to_string()))?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| TraversalModelError::BuildError(e.to_string()))?
            .with_model_from_file(onnx_model_path)
            .map_err(|e| TraversalModelError::BuildError(e.to_string()))?;

        Ok(OnnxSpeedGradeModel {
            session,
            speed_unit,
            grade_unit,
            energy_rate_unit,
        })
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use crate::routee::{
        prediction::onnx::onnx_speed_grade_model::OnnxSpeedGradeModel, prediction::PredictionModel,
    };
    use rayon::prelude::*;
    use routee_compass_core::{
        model::traversal::traversal_model_error::TraversalModelError,
        util::unit::{EnergyRate, EnergyRateUnit, Grade, GradeUnit, Speed, SpeedUnit},
    };

    #[test]
    // test that we can safely call this traversal model from multiple threads
    // since we have to implement unsafe code around the onnx runtime
    fn test_thread_saftey() {
        let model_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("routee")
            .join("test")
            .join("Toyota_Camry.onnx");
        let model: Box<dyn PredictionModel> = Box::new(
            OnnxSpeedGradeModel::new(
                &model_file_path,
                SpeedUnit::MilesPerHour,
                GradeUnit::Decimal,
                routee_compass_core::util::unit::EnergyRateUnit::GallonsGasolinePerMile,
            )
            .unwrap(),
        );

        let input_speed = Speed::new(50.0);
        let input_speed_unit = SpeedUnit::MilesPerHour;
        let input_grade = Grade::ZERO;
        let input_grade_unit = GradeUnit::Decimal;
        let input_row = (input_speed, input_speed_unit, input_grade, input_grade_unit);
        let inputs: Vec<(Speed, SpeedUnit, Grade, GradeUnit)> =
            (0..1000).map(|_i| input_row.clone()).collect();

        // map the model.get_energy function over the inputs using rayon
        let results = inputs
            .par_iter()
            .map(|(speed, speed_unit, grade, grade_unit)| {
                model.predict((*speed, *speed_unit), (*grade, *grade_unit))
            })
            .collect::<Vec<Result<(EnergyRate, EnergyRateUnit), TraversalModelError>>>();

        // assert that all of the results are Ok
        assert!(results.iter().all(|r| r.is_ok()));

        // assert that all the results are the same
        let (expected_er, expected_eru) = model
            .predict(
                (input_speed, input_speed_unit),
                (input_grade, input_grade_unit),
            )
            .unwrap();
        assert!(results.iter().all(|r| match r {
            Err(e) => panic!("{}", e),
            Ok((er, eru)) => {
                er.to_owned() == expected_er && eru.to_owned() == expected_eru
            }
        }));
    }
}
