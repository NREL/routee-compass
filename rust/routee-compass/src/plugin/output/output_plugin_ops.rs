use crate::app::{
    compass::CompassAppError,
    search::{SearchApp, SearchAppResult},
};
use routee_compass_core::algorithm::search::SearchInstance;
use serde_json::{json, Value};

/// creates the initial output with summary information from the search app,
/// which happens regardless of the output plugin setup.
pub fn create_initial_output(
    req: &Value,
    res: &Result<(SearchAppResult, SearchInstance), CompassAppError>,
    _app: &SearchApp,
) -> Result<Value, Value> {
    match &res {
        Err(e) => Err(package_error(req, e)),
        Ok((_, _)) => {
            let mut init_output = serde_json::json!({
                "request": req,
            });

            let output_plugin_executed_time = chrono::Local::now();
            init_output["output_plugin_executed_time"] =
                serde_json::json!(output_plugin_executed_time.to_rfc3339());

            Ok(init_output)
        }
    }
}

/// helper to return errors as JSON response objects which include the
/// original request along with the error message
pub fn package_error<E: ToString>(req: &Value, error: E) -> Value {
    json!({
        "request": req,
        "error": error.to_string()
    })
}
