use indoc::indoc;
use serde_json::{json, Value};
use std::rc::Rc;

use super::InputPluginError;

/// helper to return errors as JSON response objects which include the
/// original request along with the error message
pub fn package_error<E: ToString>(query: &mut Value, error: E) -> Value {
    json!({
        "request": query,
        "error": error.to_string()
    })
}

pub fn package_invariant_error(
    query: Option<&mut Value>,
    sub_section: Option<&mut Value>,
) -> Value {
    let intro = indoc! {r#"
    an input plugin has broken the invariant of the query state which requires
    that the query's JSON representation has a top-level JSON Array ([]) whose only
    elements are JSON objects ({}). please confirm that all included InputPlugin 
    instances do not break this invariant.
    "#
    };

    let json_msg = match query {
        None => String::from(
            "unable to display query state as it may have been modified during this process.",
        ),
        Some(ref q) => {
            let json_intro = "here is the invalid query state that was found:";
            let json = serde_json::to_string_pretty(q).unwrap_or_else(|e| {
                format!(
                    "oops, i can't even serialize that query because of an error: {}",
                    e
                )
            });
            format!("{}\n\n{}", json_intro, json)
        }
    };

    let msg = match sub_section {
        None => format!("{}\n{}", intro, json_msg),
        Some(ss) => {
            let ss_msg = "error triggered by the following sub-section:";
            let ss_json = serde_json::to_string_pretty(&ss).unwrap_or_else(|e| {
                format!(
                    "oops, i can't even serialize that sub-section because of an error: {}",
                    e
                )
            });

            format!("{}\n\n{}\n\n{}\n\n{}", intro, json_msg, ss_msg, ss_json)
        }
    };

    match query {
        Some(q) => package_error(q, msg),
        None => package_error(&mut json![{"error": "unable to display query"}], msg),
    }
}

pub type InputArrayOp<'a> = Rc<dyn Fn(&mut Value) -> Result<(), InputPluginError> + 'a>;

/// executes an operation on an input query. maintains the invariant that
/// input queries should always remain wrapped in a top-level JSON Array
/// so that we can perform operations like grid search, which transform a
/// single query into multiple child queries.
pub fn json_array_op<'a>(query: &'a mut Value, op: InputArrayOp<'a>) -> Result<(), Value> {
    match query {
        Value::Array(queries) => {
            for q in queries.iter_mut() {
                op(q).map_err(|e| package_error(q, e))?;
            }
            json_array_flatten_in_place(query)
        }
        other => {
            let error = package_invariant_error(None, Some(other));
            Err(error)
        }
    }
}

/// flattens the result of input processing into a response vector, ensuring
/// that the nesting and types are correct. the flatten operation effect occurs
/// in-place on the function argument via a memory swap.
pub fn json_array_flatten_in_place(result: &mut Value) -> Result<(), Value> {
    if let Value::Array(top_array) = result {
        if top_array.iter().all(|v| !v.is_array()) {
            // short circuit if there are no nested arrays
            return Ok(());
        }

        // de-nest sub-arrays into new vector
        let mut flattened: Vec<&mut Value> = vec![];
        for v1 in top_array.iter_mut() {
            match v1 {
                Value::Array(sub_array) => {
                    for v2 in sub_array.iter_mut() {
                        flattened.push(v2)
                    }
                }
                other => flattened.push(other),
            }
        }
        let mut flat_result = json![flattened];
        std::mem::swap(result, &mut flat_result);
        Ok(())
    } else {
        let error_response = package_invariant_error(Some(result), None);
        Err(error_response)
    }
}

/// flattens the result of input processing into a response vector, ensuring
/// that the nesting and types are correct. returns a new vector of values,
/// moving the valid input processing results into the new flattened vector.
pub fn json_array_flatten(result: &mut Value) -> Result<Vec<Value>, Value> {
    let mut flattened: Vec<Value> = vec![];
    if !result.is_array() {
        let error_response = package_invariant_error(Some(result), None);
        return Err(error_response);
    }
    let mut error: Option<&mut Value> = None;
    match result {
        Value::Array(sub_array) => {
            for sub_obj in sub_array.iter_mut() {
                match sub_obj {
                    Value::Object(obj) => {
                        flattened.push(json![obj]);
                    }
                    other => {
                        error = Some(other);
                    }
                }
            }
        }
        other => {
            error = Some(other);
        }
    }

    match error {
        Some(_) => {
            let error_response = package_invariant_error(None, error);
            Err(error_response)?
        }
        None => Ok(flattened),
    }
}
