use crate::model::traversal::state::state_variable::StateVar;

use super::{
    state_error::StateError, state_feature::StateFeature, update_operation::UpdateOperation,
};
use itertools::Itertools;
use std::collections::HashMap;

pub struct StateModel {
    idx_lookup: HashMap<String, usize>,
    name_lookup: Vec<String>,
    state_features: Vec<StateFeature>,
}

impl StateModel {
    /// builds a new state model from a JSON array of deserialized StateFeatures.
    /// the JSON array matches the order and size of the feature vector. downstream
    /// models such as the TraversalModel can look up features by name and retrieve
    /// the codec or unit representation in order to do state vector arithmetic.
    pub fn new(config: &serde_json::Value) -> Result<StateModel, StateError> {
        let arr = config.as_array().ok_or_else(|| {
            StateError::BuildError(String::from(
                "expected state model configuration to be a JSON array",
            ))
        })?;
        let mut idx_lookup: HashMap<String, usize> = HashMap::new();
        let mut name_lookup: Vec<String> = vec![];
        let mut state_features: Vec<StateFeature> = vec![];

        for (idx, row) in arr.iter().enumerate() {
            let feature = serde_json::from_value::<StateFeature>(row.clone()).map_err(|e| {
                StateError::BuildError(format!(
                    "unable to parse state feature row {} due to: {}",
                    idx, e
                ))
            })?;
            name_lookup.push(feature.get_feature_name());
            idx_lookup.insert(feature.get_feature_name(), idx);
            state_features.push(feature);
        }

        Ok(StateModel {
            idx_lookup,
            name_lookup,
            state_features,
        })
    }

    // pub fn get_state_vector_size(&self) -> usize {
    //     self.name_lookup.len()
    // }

    /// convenience method for state updates where the update operation
    /// is "add".
    pub fn add(
        &self,
        state: &mut [StateVar],
        name: &String,
        value: &StateVar,
    ) -> Result<(), StateError> {
        self.update_state(state, name, value, UpdateOperation::Add)
    }

    /// convenience method for state updates where the update operation
    /// is "replace".
    pub fn replace(
        &self,
        state: &mut [StateVar],
        name: &String,
        value: &StateVar,
    ) -> Result<(), StateError> {
        self.update_state(state, name, value, UpdateOperation::Replace)
    }

    /// convenience method for state updates where the update operation
    /// is "multiply".
    pub fn multiply(
        &self,
        state: &mut [StateVar],
        name: &String,
        value: &StateVar,
    ) -> Result<(), StateError> {
        self.update_state(state, name, value, UpdateOperation::Multiply)
    }

    /// performs a state update for a feature name and value by applying some
    /// update operation that handles combining the previous and next values.
    ///
    /// # Arguments
    ///
    /// * `state` - the state to update
    /// * `name`  - feature name to update
    /// * `value` - new value to apply
    /// * `op`    - operation to combine/replace prev with new value
    pub fn update_state(
        &self,
        state: &mut [StateVar],
        name: &String,
        value: &StateVar,
        op: UpdateOperation,
    ) -> Result<(), StateError> {
        let index = self.get_index(name)?;
        let prev = state
            .get(index)
            .ok_or(StateError::InvalidStateVariableIndex(index, state.len()))?;
        let updated = op.perform_operation(prev, value);
        state[index] = updated;
        Ok(())
    }

    pub fn get_names(&self) -> String {
        let names = self.idx_lookup.keys().join(", ");
        format!("[{}]", names)
    }

    fn get_index(&self, name: &String) -> Result<usize, StateError> {
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

    fn get_unit_for_index(&self, index: usize) -> Result<StateFeature, StateError> {
        self.state_features
            .get(index)
            .ok_or(StateError::InvalidStateVariableIndex(
                index,
                self.name_lookup.len(),
            ))
            .cloned()
    }

    pub fn get_unit_for_name(&self, name: &String) -> Result<StateFeature, StateError> {
        let index = self.get_index(name)?;
        self.get_unit_for_index(index)
    }
}
