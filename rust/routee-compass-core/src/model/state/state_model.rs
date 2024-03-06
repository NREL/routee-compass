use super::{
    state_error::StateError, state_feature::StateFeature, state_model_entry::StateModelEntry,
    update_operation::UpdateOperation,
};
use crate::model::traversal::state::state_variable::StateVar;
use itertools::Itertools;
use std::collections::HashMap;

pub struct StateModel(HashMap<String, StateModelEntry>);

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
        // let mut state_model: HashMap<String, StateModelEntry> = HashMap::new();
        let state_model = arr
            .iter()
            .enumerate()
            .map(|(index, row)| {
                let feature = serde_json::from_value::<StateFeature>(row.clone()).map_err(|e| {
                    StateError::BuildError(format!(
                        "unable to parse state feature row {} due to: {}",
                        index, e
                    ))
                })?;
                let feature_name = feature.get_feature_name();
                let entry = StateModelEntry { index, feature };
                Ok((feature_name, entry))
            })
            .collect::<Result<HashMap<_, _>, _>>()?;
        Ok(StateModel(state_model))
    }

    /// convenience method for state updates where the update operation
    /// is "add".
    ///
    /// * `state` - the state to update
    /// * `name`  - feature name to update
    /// * `value` - new value to apply
    pub fn update_add(
        &self,
        state: &mut [StateVar],
        name: &String,
        value: &StateVar,
    ) -> Result<(), StateError> {
        self.update_state(state, name, value, UpdateOperation::Add)
    }

    /// convenience method for state updates where the update operation
    /// is "replace".
    ///
    /// * `state` - the state to update
    /// * `name`  - feature name to update
    /// * `value` - new value to apply
    pub fn update_replace(
        &self,
        state: &mut [StateVar],
        name: &String,
        value: &StateVar,
    ) -> Result<(), StateError> {
        self.update_state(state, name, value, UpdateOperation::Replace)
    }

    /// convenience method for state updates where the update operation
    /// is "multiply".
    ///
    /// * `state` - the state to update
    /// * `name`  - feature name to update
    /// * `value` - new value to apply
    pub fn update_multiply(
        &self,
        state: &mut [StateVar],
        name: &String,
        value: &StateVar,
    ) -> Result<(), StateError> {
        self.update_state(state, name, value, UpdateOperation::Multiply)
    }

    /// convenience method for state updates where the update operation
    /// is "max".
    ///
    /// * `state` - the state to update
    /// * `name`  - feature name to update
    /// * `value` - new value to apply
    pub fn update_max(
        &self,
        state: &mut [StateVar],
        name: &String,
        value: &StateVar,
    ) -> Result<(), StateError> {
        self.update_state(state, name, value, UpdateOperation::Max)
    }

    /// convenience method for state updates where the update operation
    /// is "min".
    ///
    /// * `state` - the state to update
    /// * `name`  - feature name to update
    /// * `value` - new value to apply
    pub fn update_min(
        &self,
        state: &mut [StateVar],
        name: &String,
        value: &StateVar,
    ) -> Result<(), StateError> {
        self.update_state(state, name, value, UpdateOperation::Min)
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

    fn names_to_string(&self) -> String {
        let names = self.0.keys().join(", ");
        format!("[{}]", names)
    }

    fn get_index(&self, name: &String) -> Result<usize, StateError> {
        self.0.get(name).map(|entry| entry.index).ok_or_else(|| {
            let names = self.names_to_string();
            StateError::UnknownStateVariableName(name.into(), names)
        })
    }

    pub fn get_feature(&self, name: &String) -> Result<&StateFeature, StateError> {
        self.0.get(name).map(|entry| &entry.feature).ok_or_else(|| {
            let names = self.names_to_string();
            StateError::UnknownStateVariableName(name.into(), names)
        })
    }
}
