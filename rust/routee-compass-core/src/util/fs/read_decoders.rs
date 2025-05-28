use crate::model::state::StateVariable;
use std::fmt::Display;

/// default read decoder for an arbitrary type if the read operation
/// can be performed via the FromStr trait for type T
pub fn default<T>(_idx: usize, row: String) -> Result<T, std::io::Error>
where
    T: std::str::FromStr<Err = String>,
{
    row.parse::<T>().map_err(|e| handle_error(&row, e))
}

pub fn string(_idx: usize, row: String) -> Result<String, std::io::Error> {
    Ok(row)
}

pub fn u8(_idx: usize, row: String) -> Result<u8, std::io::Error> {
    row.parse::<u8>().map_err(|e| handle_error(&row, e))
}

pub fn f64(_idx: usize, row: String) -> Result<f64, std::io::Error> {
    row.parse::<f64>().map_err(|e| handle_error(&row, e))
}

pub fn state_variable(_idx: usize, row: String) -> Result<StateVariable, std::io::Error> {
    row.parse::<StateVariable>()
        .map_err(|e| handle_error(&row, e))
}

/// used when reading boolean files into custom feature types, stored as a Box<[StateVariable]>
/// see [`crate::model::traversal::default::custom::CustomTraversalEngine`].
pub fn bool_to_state_variable(_idx: usize, row: String) -> Result<StateVariable, std::io::Error> {
    let value = row.parse::<bool>().map_err(|e| handle_error(&row, e))?;
    match value {
        true => Ok(StateVariable(1.0)),
        false => Ok(StateVariable(0.0)),
    }
}

/// used when reading numeric i64 values into custom feature types, stored as a Box<[StateVariable]>
/// see [`crate::model::traversal::default::custom::CustomTraversalEngine`].
pub fn i64_to_state_variable(_idx: usize, row: String) -> Result<StateVariable, std::io::Error> {
    let value = row.parse::<i64>().map_err(|e| handle_error(&row, e))?;
    Ok(StateVariable(value as f64))
}

/// used when reading boolean files into custom feature types, stored as a Box<[StateVariable]>
/// see [`crate::model::traversal::default::custom::CustomTraversalEngine`].
pub fn u64_to_state_variable(_idx: usize, row: String) -> Result<StateVariable, std::io::Error> {
    let value = row.parse::<u64>().map_err(|e| handle_error(&row, e))?;
    Ok(StateVariable(value as f64))
}

/// helper to construct error messages
fn handle_error<E: Display>(row: &String, e: E) -> std::io::Error {
    let msg = format!("failure decoding row {} due to: {:}", row, e);
    std::io::Error::new(std::io::ErrorKind::InvalidData, msg)
}
