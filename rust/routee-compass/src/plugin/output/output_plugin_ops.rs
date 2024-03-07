use crate::app::{
    compass::compass_app_error::CompassAppError,
    search::{search_app::SearchApp, search_app_result::SearchAppResult},
};
use routee_compass_core::{
    algorithm::search::search_instance::SearchInstance, util::duration_extension::DurationExtension,
};
use serde_json::{json, Value};
use std::time::Duration;

/// creates the initial output with summary information from the search app,
/// which happens regardless of the output plugin setup.
pub fn create_initial_output(
    req: &Value,
    res: &Result<(SearchAppResult, SearchInstance), CompassAppError>,
    app: &SearchApp,
) -> Result<Value, Value> {
    match &res {
        Err(e) => Err(package_error(req, e)),
        Ok((result, si)) => {
            let start_time = chrono::Local::now();
            let mut init_output = serde_json::json!({
                "request": req,
            });

            let route = result.route.to_vec();

            // build and append summaries if there is a route
            if let Some(et) = route.last() {
                // build instances of traversal and cost models to compute summaries
                // let t = get_traversal_model(et, req, app)?;
                // let c = get_cost_model(et, req, app, t.clone())?;
                init_output["traversal_summary"] =
                    si.state_model.serialize_state_and_model(&et.result_state);
                let cost_summary = match si.cost_model.serialize_cost_with_info(&et.result_state) {
                    Ok(summary) => summary,
                    Err(e) => return Err(package_error(req, e)),
                };
                init_output["cost_summary"] = cost_summary;
            }

            // append the runtime required to compute these summaries
            let output_plugin_executed_time = chrono::Local::now();
            let basic_summary_runtime = output_plugin_executed_time - start_time;
            let basic_summary_runtime_str = basic_summary_runtime
                .to_std()
                .unwrap_or(Duration::ZERO)
                .hhmmss();
            init_output["basic_summary_runtime"] = serde_json::json!(basic_summary_runtime_str);
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
