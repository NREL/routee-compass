use std::path::Path;

use pyo3::{exceptions::PyException, prelude::*, types::PyType};
use routee_compass::app::compass::{
    compass_app::CompassApp, compass_app_ops::read_config_from_string,
    config::compass_app_builder::CompassAppBuilder,
};

#[pyclass]
pub struct CompassAppWrapper {
    routee_compass: CompassApp,
}

#[pymethods]
impl CompassAppWrapper {
    #[classmethod]
    pub fn _from_config_toml_string(
        _cls: &PyType,
        config_string: String,
        original_file_path: String,
    ) -> PyResult<Self> {
        let config = read_config_from_string(
            config_string.clone(),
            config::FileFormat::Toml,
            original_file_path,
        )
        .map_err(|e| {
            PyException::new_err(format!(
                "Could not create CompassApp from config string: {}",
                e
            ))
        })?;
        let builder = CompassAppBuilder::default();
        let routee_compass = CompassApp::try_from((&config, &builder)).map_err(|e| {
            PyException::new_err(format!(
                "Could not create CompassApp from config string {}: {}",
                config_string.clone(),
                e
            ))
        })?;
        Ok(CompassAppWrapper { routee_compass })
    }
    #[classmethod]
    pub fn _from_config_file(_cls: &PyType, config_file: String) -> PyResult<Self> {
        let config_path = Path::new(&config_file);
        let routee_compass = CompassApp::try_from(config_path).map_err(|e| {
            PyException::new_err(format!(
                "Could not create CompassApp from config file {}: {}",
                config_file, e
            ))
        })?;
        Ok(CompassAppWrapper { routee_compass })
    }

    /// Runs a set of queries and returns the results
    /// # Arguments
    /// * `queries` - a list of queries to run as json strings
    ///
    /// # Returns
    /// * a list of json strings containing the results of the queries
    pub fn _run_queries(&self, queries: Vec<String>) -> PyResult<Vec<String>> {
        let json_queries = queries
            .iter()
            .map(|q| serde_json::from_str(q))
            .collect::<Result<Vec<serde_json::Value>, serde_json::Error>>()
            .map_err(|e| PyException::new_err(format!("Could not parse queries: {}", e)))?;

        let results = self
            .routee_compass
            .run(json_queries)
            .map_err(|e| PyException::new_err(format!("Could not run queries: {}", e)))?;

        let string_results: Vec<String> = results.iter().map(|r| r.to_string()).collect();
        Ok(string_results)
    }
}
