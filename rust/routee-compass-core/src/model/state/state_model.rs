use super::{
    custom_feature_format::CustomFeatureFormat, state_error::StateError,
    state_feature::StateFeature, state_model_entry::StateModelEntry,
    update_operation::UpdateOperation,
};
use crate::model::{
    traversal::state::state_variable::StateVar,
    unit::{Distance, DistanceUnit, Energy, EnergyUnit, Time, TimeUnit},
};
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
    ///   { "distance_unit" = "kilometers", initial = 0.0 },
    ///   { "time_unit" = "minutes", initial = 0.0 },
    ///   { name = "soc", unit = "percent", format = { type = "floating_point", initial = 0.0 } }
    /// ]
    ///
    /// the same example as JSON (convert '=' into ':', and enquote object keys):
    ///
    /// ```json
    /// [
    ///   { "distance_unit": "kilometers", "initial": 0.0 },
    ///   { "time_unit": "minutes", "initial": 0.0 },
    ///   {
    ///     "name": "soc",
    ///     "unit": "percent",
    ///     "format": {
    ///       "type": "floating_point",
    ///       "initial": 0.0
    ///     }
    ///   }
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

    /// retrieves a state variable that is expected to have a type of Distance
    ///
    /// # Arguments
    /// * `state` - state vector to inspect
    /// * `name`  - feature name to extract
    /// * `unit`  - feature is converted to this unit before returning
    ///
    /// # Returns
    ///
    /// feature value in the expected unit type, or an error
    pub fn get_distance(
        &self,
        state: &[StateVar],
        name: &str,
        unit: &DistanceUnit,
    ) -> Result<Distance, StateError> {
        let value = self.get_value(state, name)?;
        let feature = self.get_feature(name)?;
        let result = feature.get_distance_unit()?.convert(&value.into(), unit);
        Ok(result)
    }
    /// retrieves a state variable that is expected to have a type of Time
    ///
    /// # Arguments
    /// * `state` - state vector to inspect
    /// * `name`  - feature name to extract
    /// * `unit`  - feature is converted to this unit before returning
    ///
    /// # Returns
    ///
    /// feature value in the expected unit type, or an error
    pub fn get_time(
        &self,
        state: &[StateVar],
        name: &str,
        unit: &TimeUnit,
    ) -> Result<Time, StateError> {
        let value = self.get_value(state, name)?;
        let feature = self.get_feature(name)?;
        let result = feature.get_time_unit()?.convert(&value.into(), unit);
        Ok(result)
    }
    /// retrieves a state variable that is expected to have a type of Energy
    ///
    /// # Arguments
    /// * `state` - state vector to inspect
    /// * `name`  - feature name to extract
    /// * `unit`  - feature is converted to this unit before returning
    ///
    /// # Returns
    ///
    /// feature value in the expected unit type, or an error
    pub fn get_energy(
        &self,
        state: &[StateVar],
        name: &str,
        unit: &EnergyUnit,
    ) -> Result<Energy, StateError> {
        let value = self.get_value(state, name)?;
        let feature = self.get_feature(name)?;
        let result = feature.get_energy_unit()?.convert(&value.into(), unit);
        Ok(result)
    }
    /// retrieves a state variable that is expected to have a type of f64.
    ///
    /// # Arguments
    /// * `state` - state vector to inspect
    /// * `name`  - feature name to extract
    ///
    /// # Returns
    ///
    /// the expected value or an error
    pub fn get_custom_f64(&self, state: &[StateVar], name: &str) -> Result<f64, StateError> {
        let (value, format) = self.get_custom_state_variable(state, name)?;
        let result = format.decode_f64(&value)?;
        Ok(result)
    }
    /// retrieves a state variable that is expected to have a type of i64.
    ///
    /// # Arguments
    /// * `state` - state vector to inspect
    /// * `name`  - feature name to extract
    ///
    /// # Returns
    ///
    /// the expected value or an error
    pub fn get_custom_i64(&self, state: &[StateVar], name: &str) -> Result<i64, StateError> {
        let (value, format) = self.get_custom_state_variable(state, name)?;
        let result = format.decode_i64(&value)?;
        Ok(result)
    }
    /// retrieves a state variable that is expected to have a type of u64.
    ///
    /// # Arguments
    /// * `state` - state vector to inspect
    /// * `name`  - feature name to extract
    ///
    /// # Returns
    ///
    /// the expected value or an error
    pub fn get_custom_u64(&self, state: &[StateVar], name: &str) -> Result<u64, StateError> {
        let (value, format) = self.get_custom_state_variable(state, name)?;
        let result = format.decode_u64(&value)?;
        Ok(result)
    }
    /// retrieves a state variable that is expected to have a type of bool.
    ///
    /// # Arguments
    /// * `state` - state vector to inspect
    /// * `name`  - feature name to extract
    ///
    /// # Returns
    ///
    /// the expected value or an error
    pub fn get_custom_bool(&self, state: &[StateVar], name: &str) -> Result<bool, StateError> {
        let (value, format) = self.get_custom_state_variable(state, name)?;
        let result = format.decode_bool(&value)?;
        Ok(result)
    }

    /// internal helper function that retrieves a value as a feature vector state variable
    /// along with the custom feature's format. this is used by the four specialized get_custom
    /// methods for specific types.
    ///
    /// # Arguments
    /// * `state` - state vector to inspect
    /// * `name`  - feature name to extract
    ///
    /// # Returns
    ///
    /// the expected value as a state variable (not decoded) or an error
    fn get_custom_state_variable(
        &self,
        state: &[StateVar],
        name: &str,
    ) -> Result<(StateVar, &CustomFeatureFormat), StateError> {
        let value = self.get_value(state, name)?;
        let feature = self.get_feature(name)?;
        let format = feature.get_custom_feature_format()?;
        Ok((value, format))
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

    pub fn add_distance(
        &self,
        state: &mut [StateVar],
        name: &str,
        edge_distance: &Distance,
        from_unit: &DistanceUnit,
    ) -> Result<(), StateError> {
        let prev_distance = self.get_distance(state, name, from_unit)?;
        let next_distance = prev_distance + *edge_distance;
        self.set_distance(state, "time", &next_distance, from_unit)
    }

    pub fn add_time(
        &self,
        state: &mut [StateVar],
        name: &str,
        edge_time: &Time,
        from_unit: &TimeUnit,
    ) -> Result<(), StateError> {
        let prev_time = self.get_time(state, name, from_unit)?;
        let next_time = prev_time + *edge_time;
        self.set_time(state, "time", &next_time, from_unit)
    }

    pub fn add_energy(
        &self,
        state: &mut [StateVar],
        name: &str,
        edge_energy: &Energy,
        from_unit: &EnergyUnit,
    ) -> Result<(), StateError> {
        let prev_energy = self.get_energy(state, name, from_unit)?;
        let next_energy = prev_energy + *edge_energy;
        self.set_energy(state, "time", &next_energy, from_unit)
    }

    pub fn set_distance(
        &self,
        state: &mut [StateVar],
        name: &str,
        distance: &Distance,
        from_unit: &DistanceUnit,
    ) -> Result<(), StateError> {
        let feature = self.get_feature(name)?;
        let to_unit = feature.get_distance_unit()?;
        let value = from_unit.convert(distance, &to_unit);
        self.update_state(state, name, &value.into(), UpdateOperation::Replace)
    }

    pub fn set_time(
        &self,
        state: &mut [StateVar],
        name: &str,
        time: &Time,
        from_unit: &TimeUnit,
    ) -> Result<(), StateError> {
        let feature = self.get_feature(name)?;
        let to_unit = feature.get_time_unit()?;
        let value = from_unit.convert(time, &to_unit);
        self.update_state(state, name, &value.into(), UpdateOperation::Replace)
    }

    pub fn set_energy(
        &self,
        state: &mut [StateVar],
        name: &str,
        energy: &Energy,
        from_unit: &EnergyUnit,
    ) -> Result<(), StateError> {
        let feature = self.get_feature(name)?;
        let to_unit = feature.get_energy_unit()?;
        let value = from_unit.convert(energy, &to_unit);
        self.update_state(state, name, &value.into(), UpdateOperation::Replace)
    }

    pub fn set_custom_f64(
        &self,
        state: &mut [StateVar],
        name: &str,
        value: &f64,
    ) -> Result<(), StateError> {
        let feature = self.get_feature(name)?;
        let format = feature.get_custom_feature_format()?;
        let encoded_value = format.encode_f64(value)?;
        self.update_state(state, name, &encoded_value, UpdateOperation::Replace)
    }

    pub fn set_custom_i64(
        &self,
        state: &mut [StateVar],
        name: &str,
        value: &i64,
    ) -> Result<(), StateError> {
        let feature = self.get_feature(name)?;
        let format = feature.get_custom_feature_format()?;
        let encoded_value = format.encode_i64(value)?;
        self.update_state(state, name, &encoded_value, UpdateOperation::Replace)
    }

    pub fn set_custom_u64(
        &self,
        state: &mut [StateVar],
        name: &str,
        value: &u64,
    ) -> Result<(), StateError> {
        let feature = self.get_feature(name)?;
        let format = feature.get_custom_feature_format()?;
        let encoded_value = format.encode_u64(value)?;
        self.update_state(state, name, &encoded_value, UpdateOperation::Replace)
    }

    pub fn set_custom_bool(
        &self,
        state: &mut [StateVar],
        name: &str,
        value: &bool,
    ) -> Result<(), StateError> {
        let feature = self.get_feature(name)?;
        let format = feature.get_custom_feature_format()?;
        let encoded_value = format.encode_bool(value)?;
        self.update_state(state, name, &encoded_value, UpdateOperation::Replace)
    }

    // /// convenience method for state updates where the update operation
    // /// is "add".
    // ///
    // /// # Arguments
    // ///
    // /// * `state` - the state to update
    // /// * `name`  - feature name to update
    // /// * `value` - new value to apply
    // fn update_add(
    //     &self,
    //     state: &mut [StateVar],
    //     name: &str,
    //     value: &StateVar,
    // ) -> Result<(), StateError> {
    //     self.update_state(state, name, value, UpdateOperation::Add)
    // }

    // /// convenience method for state updates where the update operation
    // /// is "replace".
    // ///
    // /// * `state` - the state to update
    // /// * `name`  - feature name to update
    // /// * `value` - new value to apply
    // fn update_replace(
    //     &self,
    //     state: &mut [StateVar],
    //     name: &str,
    //     value: &StateVar,
    // ) -> Result<(), StateError> {
    //     self.update_state(state, name, value, UpdateOperation::Replace)
    // }

    // /// convenience method for state updates where the update operation
    // /// is "multiply".
    // ///
    // /// * `state` - the state to update
    // /// * `name`  - feature name to update
    // /// * `value` - new value to apply
    // fn update_multiply(
    //     &self,
    //     state: &mut [StateVar],
    //     name: &str,
    //     value: &StateVar,
    // ) -> Result<(), StateError> {
    //     self.update_state(state, name, value, UpdateOperation::Multiply)
    // }

    // /// convenience method for state updates where the update operation
    // /// is "max".
    // ///
    // /// * `state` - the state to update
    // /// * `name`  - feature name to update
    // /// * `value` - new value to apply
    // fn update_max(
    //     &self,
    //     state: &mut [StateVar],
    //     name: &str,
    //     value: &StateVar,
    // ) -> Result<(), StateError> {
    //     self.update_state(state, name, value, UpdateOperation::Max)
    // }

    // /// convenience method for state updates where the update operation
    // /// is "min".
    // ///
    // /// * `state` - the state to update
    // /// * `name`  - feature name to update
    // /// * `value` - new value to apply
    // fn update_min(
    //     &self,
    //     state: &mut [StateVar],
    //     name: &str,
    //     value: &StateVar,
    // ) -> Result<(), StateError> {
    //     self.update_state(state, name, value, UpdateOperation::Min)
    // }

    // /// convenience method for state updates where the update operation
    // /// is "add".
    // ///
    // /// # Arguments
    // ///
    // /// * `state` - the state to update
    // /// * `name`  - feature name to update
    // /// * `value` - new value to apply
    // fn update_add_bounded(
    //     &self,
    //     state: &mut [StateVar],
    //     name: &str,
    //     value: &StateVar,
    //     min: &StateVar,
    //     max: &StateVar,
    // ) -> Result<(), StateError> {
    //     self.update_state(state, name, value, UpdateOperation::AddBounded(*min, *max))
    // }

    /// performs a state update for a feature name and value by applying some
    /// update operation that handles combining the previous and next values.
    ///
    /// # Arguments
    ///
    /// * `state` - the state to update
    /// * `name`  - feature name to update
    /// * `value` - new value to apply
    /// * `op`    - operation to combine/replace prev with new value
    fn update_state(
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

    fn get_value(&self, state: &[StateVar], name: &str) -> Result<StateVar, StateError> {
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
                "unable to parse state feature row {} with contents '{}' due to: {}",
                index,
                row.clone(),
                e
            ))
        })?;
        let feature_name = feature.get_feature_name();
        let entry = StateModelEntry { index, feature };
        Ok((feature_name, entry))
    });
    Ok(iterator)
}
