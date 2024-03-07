use super::{
    state_error::StateError, state_feature::StateFeature, state_model_entry::StateModelEntry,
    update_operation::UpdateOperation,
};
use crate::model::traversal::state::state_variable::StateVar;
use itertools::Itertools;
use serde_json::json;
use std::collections::HashMap;

pub struct StateModel(HashMap<String, StateModelEntry>);

impl StateModel {
    /// builds a new state model from a JSON array of deserialized StateFeatures.
    /// the JSON array matches the order and size of the feature vector. downstream
    /// models such as the TraversalModel can look up features by name and retrieve
    /// the codec or unit representation in order to do state vector arithmetic.
    ///
    /// # Example
    ///
    /// ### Deserialization
    ///
    /// an example TOML representation of a StateModel:
    ///
    /// ```toml
    /// [
    ///   { "distance_unit" = "kilometers" },
    ///   { "time_unit" = "minutes" },
    ///   { custom_feature_name = "soc", codec = { type = "floating_point", initial = 0.0 } }
    /// ]
    ///
    /// the same example as JSON (convert '=' into ':', and enquote object keys):
    ///
    /// ```json
    /// [
    ///   { "distance_unit": "kilometers" },
    ///   { "time_unit": "minutes" },
    ///   { "custom_feature_name": "soc", "codec": { "type": "floating_point", "initial": 0.0 } }
    /// ]
    /// ```
    pub fn new(config: &serde_json::Value) -> Result<StateModel, StateError> {
        let state_model = rows_from_json(config)?.collect::<Result<HashMap<_, _>, _>>()?;
        Ok(StateModel(state_model))
    }

    pub fn empty() -> StateModel {
        StateModel(HashMap::new())
    }

    /// extends a state model by adding additional key/value pairs to the model mapping.
    /// in the case of name collision, a warning is logged to the user.
    ///
    /// this method is used when state models are updated by the user query as Services
    /// become Models in the SearchApp.
    ///
    /// # Arguments
    /// * `query` - JSON search query contents containing state model information
    pub fn extend(&self, entries: Vec<(String, StateFeature)>) -> Result<StateModel, StateError> {
        let offset = self.len();
        let mut map = self.0.clone();
        for row in entries.iter().enumerate() {
            let (i, (name, feature)) = row;
            let index = offset + i;
            let entry = StateModelEntry {
                index,
                feature: feature.clone(),
            };
            let insert_result = map.insert(name.clone(), entry);
            if let Some(replaced) = insert_result {
                log::warn!(
                    "user overwriting state model entry {} with {}",
                    name,
                    replaced
                );
            }
        }
        Ok(StateModel(map))
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// generates an iterator presenting the state model entries sorted
    /// by index, so that it may be zipped with a real state vector
    pub fn state_model_iterator(&self) -> impl Iterator<Item = (&String, &StateModelEntry)> {
        self.0.iter().sorted_by_key(|(_n, f)| f.index)
    }

    /// collects the state model tuples and clones them so they can
    /// be used to build other collections
    pub fn state_model_vec(&self) -> Vec<(String, StateModelEntry)> {
        self.state_model_iterator()
            .map(|(n, e)| (n.clone(), e.clone()))
            .collect_vec()
    }

    /// Creates the initial state of a search. this should be a vector of
    /// accumulators, defined in the state model configuration.
    ///
    /// # Returns
    ///
    /// an initialized, "zero"-valued traversal state, or an error
    pub fn initial_state(&self) -> Result<Vec<StateVar>, StateError> {
        self.state_model_iterator()
            .map(|(n, _idx)| {
                let feature = self.get_feature(n)?;
                let initial = feature.get_initial()?;
                Ok(initial)
            })
            .collect::<Result<Vec<_>, _>>()
    }

    /// gets a value from a state by name.
    ///
    /// # Arguments
    ///
    /// * `state` - the state to inspect
    /// * `name`  - name of feature to extract
    ///
    /// # Result
    ///
    /// the requested state variable or an error on failure
    pub fn get_value(&self, state: &[StateVar], name: &str) -> Result<StateVar, StateError> {
        let idx = self.get_index(name)?;
        if idx >= state.len() {
            Err(StateError::RuntimeError(format!(
                "state index {} for {} is out of range for state vector with {} entries",
                idx,
                name,
                state.len()
            )))
        } else {
            Ok(state[idx])
        }
    }

    /// gets the difference from some previous value to some next value by name.
    ///
    /// # Arguments
    ///
    /// * `prev` - the previous state to inspect
    /// * `next` - the next state to inspect
    /// * `name`  - name of feature to compare
    ///
    /// # Result
    ///
    /// the delta between states for this variable, or an error
    pub fn get_delta(
        &self,
        prev: &[StateVar],
        next: &[StateVar],
        name: &str,
    ) -> Result<StateVar, StateError> {
        let prev_val = self.get_value(prev, name)?;
        let next_val = self.get_value(next, name)?;
        Ok(next_val - prev_val)
    }

    /// convenience method for state updates where the update operation
    /// is "add".
    ///
    /// # Arguments
    ///
    /// * `state` - the state to update
    /// * `name`  - feature name to update
    /// * `value` - new value to apply
    pub fn update_add(
        &self,
        state: &mut [StateVar],
        name: &str,
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
        name: &str,
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
        name: &str,
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
        name: &str,
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
        name: &str,
        value: &StateVar,
    ) -> Result<(), StateError> {
        self.update_state(state, name, value, UpdateOperation::Min)
    }

    /// convenience method for state updates where the update operation
    /// is "add".
    ///
    /// # Arguments
    ///
    /// * `state` - the state to update
    /// * `name`  - feature name to update
    /// * `value` - new value to apply
    pub fn update_add_bounded(
        &self,
        state: &mut [StateVar],
        name: &str,
        value: &StateVar,
        min: &StateVar,
        max: &StateVar,
    ) -> Result<(), StateError> {
        self.update_state(state, name, value, UpdateOperation::AddBounded(*min, *max))
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
        name: &str,
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

    pub fn serialize_state(&self, state: &[StateVar]) -> serde_json::Value {
        let output = self
            .state_model_iterator()
            .zip(state.iter())
            .map(|((name, _), state_var)| (name, state_var))
            .collect::<HashMap<_, _>>();
        json![output]
    }

    pub fn serialize_state_model(&self) -> serde_json::Value {
        json![self.0]
    }

    pub fn serialize_state_and_model(&self, state: &[StateVar]) -> serde_json::Value {
        let mut summary = self.serialize_state(state);
        if let serde_json::Value::Object(m) = self.serialize_state_model() {
            for (k, v) in m.into_iter() {
                summary[k] = v;
            }
        }

        summary
    }

    fn names_to_string(&self) -> String {
        let names = self.0.keys().join(", ");
        format!("[{}]", names)
    }

    fn get_index(&self, name: &str) -> Result<usize, StateError> {
        self.0.get(name).map(|entry| entry.index).ok_or_else(|| {
            let names = self.names_to_string();
            StateError::UnknownStateVariableName(name.into(), names)
        })
    }

    pub fn get_feature(&self, name: &str) -> Result<&StateFeature, StateError> {
        self.0.get(name).map(|entry| &entry.feature).ok_or_else(|| {
            let names = self.names_to_string();
            StateError::UnknownStateVariableName(name.into(), names)
        })
    }
}

fn rows_from_json(
    json: &serde_json::Value,
) -> Result<impl Iterator<Item = Result<(String, StateModelEntry), StateError>> + '_, StateError> {
    let arr = json.as_array().ok_or_else(|| {
        StateError::BuildError(String::from(
            "expected state model configuration to be a JSON array",
        ))
    })?;

    let iterator = arr.iter().enumerate().map(|(index, row)| {
        let feature = serde_json::from_value::<StateFeature>(row.clone()).map_err(|e| {
            StateError::BuildError(format!(
                "unable to parse state feature row {} due to: {}",
                index, e
            ))
        })?;
        let feature_name = feature.get_feature_name();
        let entry = StateModelEntry { index, feature };
        Ok((feature_name, entry))
    });
    Ok(iterator)
}
