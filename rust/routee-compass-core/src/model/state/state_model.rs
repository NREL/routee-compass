use super::{state_error::StateError, state_variable_unit::StateVariableUnit};
use itertools::Itertools;
use std::collections::HashMap;

pub struct StateModel {
    idx_lookup: HashMap<String, usize>,
    name_lookup: Vec<String>,
    units: Vec<StateVariableUnit>,
}

impl StateModel {
    pub fn get_state_vector_size(&self) -> usize {
        self.name_lookup.len()
    }

    pub fn get_names(&self) -> String {
        let names = self.idx_lookup.keys().join(", ");
        format!("[{}]", names)
    }

    pub fn get_index(&self, name: &String) -> Result<usize, StateError> {
        self.idx_lookup
            .get(name)
            .ok_or_else(|| {
                let names = self.get_names();
                StateError::UnknownStateVariableName(name.into(), names)
            })
            .cloned()
    }

    pub fn get_name(&self, index: usize) -> Result<String, StateError> {
        self.name_lookup
            .get(index)
            .cloned()
            .ok_or(StateError::InvalidStateVariableIndex(
                index,
                self.name_lookup.len(),
            ))
    }

    pub fn get_unit_for_index(&self, index: usize) -> Result<StateVariableUnit, StateError> {
        self.units
            .get(index)
            .ok_or(StateError::InvalidStateVariableIndex(
                index,
                self.name_lookup.len(),
            ))
            .cloned()
    }

    pub fn get_unit_for_name(&self, name: &String) -> Result<StateVariableUnit, StateError> {
        let index = self.get_index(name)?;
        self.get_unit_for_index(index)
    }
}
