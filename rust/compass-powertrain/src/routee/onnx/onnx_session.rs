use compass_core::model::traversal::traversal_model_error::TraversalModelError;
use onnxruntime::{environment::Environment, session::Session, GraphOptimizationLevel};
use std::cell::UnsafeCell;

pub struct OnnxSession {
    _env: Environment,
    session: UnsafeCell<Session<'static>>,
}

impl TryFrom<String> for OnnxSession {
    type Error = TraversalModelError;

    fn try_from(filepath: String) -> Result<Self, Self::Error> {
        let environment = Environment::builder()
            .build()
            .map_err(|e| TraversalModelError::BuildError(e.to_string()))?;
        let session = unsafe {
            UnsafeCell::new(std::mem::transmute::<_, Session<'static>>(
                environment
                    .new_session_builder()
                    .map_err(|e| TraversalModelError::BuildError(e.to_string()))?
                    .with_number_threads(1)
                    .map_err(|e| TraversalModelError::BuildError(e.to_string()))?
                    .with_optimization_level(GraphOptimizationLevel::Basic)
                    .map_err(|e| TraversalModelError::BuildError(e.to_string()))?
                    .with_model_from_file(filepath)
                    .map_err(|e| TraversalModelError::BuildError(e.to_string()))?,
            ))
        };

        Ok(OnnxSession {
            _env: environment,
            session,
        })
    }
}

impl OnnxSession {
    pub fn get_session(&self) -> &mut Session<'static> {
        unsafe { &mut *self.session.get() }
    }
}

unsafe impl Send for OnnxSession {}
unsafe impl Sync for OnnxSession {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_load() {
        let model_file_path: String = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("routee")
            .join("test")
            .join("Toyota_Camry.onnx")
            .to_str()
            .unwrap()
            .into();
        let model = OnnxSession::try_from(model_file_path).unwrap();
    }
}
