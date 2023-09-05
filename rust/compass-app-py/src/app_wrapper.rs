use std::path::PathBuf;

use compass_app::app::compass::compass_app::CompassApp;
use pyo3::{exceptions::PyException, prelude::*, types::PyType};

#[pyclass]
pub struct CompassAppWrapper {
    compass_app: CompassApp,
}

#[pymethods]
impl CompassAppWrapper {
    #[classmethod]
    pub fn _from_config_file(_cls: &PyType, config_file: String) -> PyResult<Self> {
        let config_path = PathBuf::from(config_file.clone());
        let compass_app = CompassApp::try_from(config_path).map_err(|e| {
            PyException::new_err(format!(
                "Could not create CompassApp from config file {}: {}",
                config_file, e
            ))
        })?;
        Ok(CompassAppWrapper { compass_app })
    }

    /// Runs a single query and returns the result
    ///
    /// # Arguments
    ///
    /// * `query` - a json string containing the query to run
    ///
    /// # Returns
    ///
    /// * a json string containing the result of the query
    pub fn _run_query(&self, query: String) -> PyResult<String> {
        let json_query = serde_json::from_str(&query)
            .map_err(|e| PyException::new_err(format!("Could not parse query: {}", e)))?;

        let result = self
            .compass_app
            .run(vec![json_query])
            .map_err(|e| PyException::new_err(format!("Could not run query: {}", e)))?;

        Ok(result[0].to_string())
    }

    /// Runs a set of queries and returns the results
    /// # Arguments
    /// * `queries` - a list of queries to run as json strings
    ///
    /// # Returns
    /// * a list of json strings containing the results of the queries
    pub fn _run_queries(&self, queries: Vec<String>) -> PyResult<Vec<String>> {
        queries.iter().map(|q| self._run_query(q.clone())).collect()
    }
}
