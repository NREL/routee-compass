use std::{cell::UnsafeCell, path::Path, sync::Arc};

use compass_core::{
    model::{
        cost::cost::Cost,
        property::{edge::Edge, vertex::Vertex},
        traversal::{
            default::velocity_lookup::VelocityLookupModel,
            state::{state_variable::StateVar, traversal_state::TraversalState},
            traversal_model::TraversalModel,
            traversal_model_error::TraversalModelError,
            traversal_result::TraversalResult,
        },
        units::{EnergyUnit, TimeUnit},
    },
    util::geo::haversine::coord_distance_km,
};
use onnxruntime::{
    environment::Environment, session::Session, tensor::OrtOwnedTensor, GraphOptimizationLevel,
};
use uom::si;

pub struct SessionWrapper {
    env: Environment,
    session: UnsafeCell<Session<'static>>,
}

impl SessionWrapper {
    pub fn from_file<P: AsRef<Path>>(filepath: P) -> Result<Self, TraversalModelError> {
        let environment = Environment::builder()
            .build()
            .map_err(|e| TraversalModelError::BuildError(e.to_string()))?;
        let session = unsafe {
            UnsafeCell::new(std::mem::transmute::<_, Session<'static>>(
                environment
                    .new_session_builder()
                    .map_err(|e| TraversalModelError::BuildError(e.to_string()))?
                    .with_optimization_level(GraphOptimizationLevel::Basic)
                    .map_err(|e| TraversalModelError::BuildError(e.to_string()))?
                    .with_model_from_file(filepath)
                    .map_err(|e| TraversalModelError::BuildError(e.to_string()))?,
            ))
        };

        Ok(SessionWrapper {
            env: environment,
            session: session,
        })
    }

    fn get_session(&self) -> &mut Session<'static> {
        unsafe { &mut *self.session.get() }
    }

    pub fn get_energy(
        &self,
        speed_mph: f64,
        grade_percent: f64,
        distance_miles: f64,
    ) -> Result<f64, TraversalModelError> {
        let x = ndarray::Array1::from(vec![speed_mph as f32, grade_percent as f32])
            .into_shape((1, 2))
            .map_err(|e| {
                TraversalModelError::PredictionModel(format!(
                    "Failed to reshape input for prediction: {}",
                    e.to_string()
                ))
            })?;
        let input_tensor = vec![x];

        let session = self.get_session();

        let outputs: Vec<OrtOwnedTensor<f32, _>> = session
            .run(input_tensor)
            .map_err(|e| TraversalModelError::PredictionModel(e.to_string()))?;

        let energy_per_mile = outputs[0].to_owned().into_raw_vec()[0] as f64;
        let mut energy = energy_per_mile * distance_miles;
        energy = if energy < 0.0 { 0.0 } else { energy };
        Ok(energy)
    }
}

unsafe impl Send for SessionWrapper {}
unsafe impl Sync for SessionWrapper {}

pub struct RouteEOnnxModel {
    pub energy_unit: EnergyUnit,

    session: SessionWrapper,
    velocity_model: Arc<VelocityLookupModel>,
}

impl RouteEOnnxModel {
    pub fn from_file<P: AsRef<Path>>(
        onnx_path: P,
        velocity_path: P,
        time_unit: TimeUnit,
        energy_rate_unit: EnergyUnit,
    ) -> Result<Self, TraversalModelError> {
        let session_wrapper = SessionWrapper::from_file(onnx_path)?;

        let velocity_model = Arc::new(VelocityLookupModel::from_file(&velocity_path, time_unit)?);

        // sweep a fixed set of speed and grade values to find the minimum energy per mile rate from the incoming rf model
        let mut minimum_energy_per_mile = std::f64::MAX;

        for speed_mph in 1..100 {
            for grade_percent in -20..20 {
                let energy_per_mile = session_wrapper.get_energy(
                    speed_mph as f64,
                    grade_percent as f64,
                    1.0,
                )?;
                if energy_per_mile < minimum_energy_per_mile {
                    minimum_energy_per_mile = energy_per_mile;
                }
            }
        }

        Ok(Self {
            energy_unit: energy_rate_unit,
            session: session_wrapper,
            velocity_model,
        })
    }
    fn summary(&self, state: &TraversalState) -> serde_json::Value {
        let total_energy = state[0].0;
        let energy_units = match self.energy_unit {
            EnergyUnit::GallonsGasoline => "gallons_gasoline",
        };
        serde_json::json!({
            "total_energy": total_energy,
            "energy_units": energy_units
        })
    }
}

unsafe impl Send for RouteEOnnxModel {}
unsafe impl Sync for RouteEOnnxModel {}

impl TraversalModel for RouteEOnnxModel {
    fn initial_state(&self) -> TraversalState {
        vec![StateVar(0.0)]
    }

    fn traversal_cost(
        &self,
        src: &Vertex,
        edge: &Edge,
        dst: &Vertex,
        state: &TraversalState,
    ) -> Result<TraversalResult, TraversalModelError> {
        let time_result = self.velocity_model.traversal_cost(src, edge, dst, state)?;
        let time_hours: f64 = match self.velocity_model.output_unit {
            TimeUnit::Hours => time_result.total_cost.into(),
            TimeUnit::Seconds => {
                let time_seconds: f64 = time_result.total_cost.into();
                time_seconds / 3600.0
            }
            TimeUnit::Milliseconds => {
                let time_milliseconds: f64 = time_result.total_cost.into();
                time_milliseconds / 3600000.0
            }
        };
        let distance = edge.distance;
        let grade = edge.grade;
        let distance_mile = distance.get::<si::length::mile>();
        let grade_percent = grade.get::<si::ratio::percent>();
        let speed_mph = distance_mile / time_hours;

        let energy_cost = self.session.get_energy(speed_mph, grade_percent, distance_mile)?;

        let mut updated_state = state.clone();
        updated_state[0] = state[0] + StateVar(energy_cost);
        let result = TraversalResult {
            total_cost: Cost::from(energy_cost),
            updated_state,
        };
        Ok(result)
    }

    fn cost_estimate(
        &self,
        src: &Vertex,
        dst: &Vertex,
        _state: &TraversalState,
    ) -> Result<Cost, TraversalModelError> {
        let distance = coord_distance_km(src.coordinate, dst.coordinate)
            .map_err(TraversalModelError::NumericError)?;
        let distance_miles = distance.get::<si::length::mile>();
        let minimum_energy = match self.energy_unit {
            EnergyUnit::GallonsGasoline => distance_miles * (1.0 / 60.0),
        };
        Ok(Cost::from(minimum_energy))
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    use rayon::prelude::*;

    fn test_model() -> RouteEOnnxModel {
        let model_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("routee")
            .join("test")
            .join("Toyota_Camry.onnx");
        let velocity_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("routee")
            .join("test")
            .join("velocities.txt");
        let model = RouteEOnnxModel::from_file(
            model_file_path,
            velocity_file_path,
            TimeUnit::Seconds,
            EnergyUnit::GallonsGasoline,
        )
        .unwrap();
        model
    }

    #[test]
    fn test_load() {
        let model = test_model();

        let energy = model.session.get_energy(60.0, 0.0, 1.0).expect("Failed to get energy");

        println!("mpg: {}", 1.0 / energy);
    }

    #[test]
    // test that we can safely call this traversal model from multiple threads
    // since we have to implement unsafe code around the onnx runtime
    fn test_thread_saftey() {
        let model = test_model();
        let inputs: Vec<(f64, f64, f64)> = (0..1000)
            .map(|i| (50.0, 0.0, i as f64))
            .collect();
        
        // map the model.get_energy function over the inputs using rayon
        let results = inputs.par_iter().map(|(speed, grade, distance)| {
            model.session.get_energy(*speed, *grade, *distance)
        }).collect::<Vec<Result<f64, TraversalModelError>>>();

        // assert that all of the results are Ok
        assert!(results.iter().all(|r| r.is_ok()));

    }
}
