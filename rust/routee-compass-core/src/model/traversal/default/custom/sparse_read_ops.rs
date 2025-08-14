use crate::{
    model::{network::EdgeId, state::StateVariable, traversal::TraversalModelError},
    util::fs::read_utils,
};
use kdam::BarBuilder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// makes a conversion from boolean values ("true", "false") to floating-point values (1.0, 0.0)
pub fn read_bools(
    file: &String,
    bar_builder: BarBuilder,
) -> Result<HashMap<EdgeId, StateVariable>, TraversalModelError> {
    let rows =
        read_utils::from_csv::<BoolRow>(&file, true, Some(bar_builder), None).map_err(|e| {
            TraversalModelError::BuildError(format!(
                "failure reading custom input file {file}: {e}"
            ))
        })?;
    let values = rows
        .iter()
        .map(|row| (row.edge_id, StateVariable(row.to_f64())))
        .collect::<HashMap<_, _>>();
    Ok(values)
}

/// since (numeric ranges practical for Compass) i64 and u64 can be expressed relatively well via f64,
/// we can deserialize values of i64 and u64 type directly as StateVariable (f64) values.
pub fn read_state_variables(
    file: &String,
    bar_builder: BarBuilder,
) -> Result<HashMap<EdgeId, StateVariable>, TraversalModelError> {
    let rows =
        read_utils::from_csv::<F64Row>(&file, true, Some(bar_builder), None).map_err(|e| {
            TraversalModelError::BuildError(format!(
                "failure reading custom input file {file}: {e}"
            ))
        })?;
    let values = rows
        .iter()
        .map(|row| (row.edge_id, row.value))
        .collect::<HashMap<_, _>>();
    Ok(values)
}

#[derive(Serialize, Deserialize)]
struct F64Row {
    edge_id: EdgeId,
    value: StateVariable,
}

#[derive(Serialize, Deserialize)]
struct BoolRow {
    edge_id: EdgeId,
    value: bool,
}

impl BoolRow {
    pub fn to_f64(&self) -> f64 {
        self.value as u8 as f64
    }
}
