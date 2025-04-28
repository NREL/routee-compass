use super::StateVariable;
use super::{
    custom_feature_format::CustomFeatureFormat, state_feature::StateFeature,
    state_model_error::StateModelError, update_operation::UpdateOperation,
};
use crate::model::unit::Convert;
use crate::util::compact_ordered_hash_map::CompactOrderedHashMap;
use crate::{
    model::unit::{Distance, DistanceUnit, Energy, EnergyUnit, Time, TimeUnit},
    util::compact_ordered_hash_map::IndexedEntry,
};
use itertools::Itertools;
use serde_json::json;
use std::borrow::Cow;
use std::collections::HashMap;
use std::iter::Enumerate;

/// a state model tracks information about each feature in a search state vector.
/// in concept, it is modeled as a mapping from a feature_name String to a StateFeature
/// object (see NFeatures, below). there are 4 additional implementations that specialize
/// for the case where fewer than 5 features are required in order to improve CPU performance.
pub struct StateModel(CompactOrderedHashMap<String, StateFeature>);
type FeatureIterator<'a> = Box<dyn Iterator<Item = (&'a String, &'a StateFeature)> + 'a>;
type IndexedFeatureIterator<'a> =
    Enumerate<Box<dyn Iterator<Item = (&'a String, &'a StateFeature)> + 'a>>;

impl StateModel {
    pub fn new(features: Vec<(String, StateFeature)>) -> StateModel {
        let map = CompactOrderedHashMap::new(features);
        StateModel(map)
    }

    pub fn empty() -> StateModel {
        StateModel(CompactOrderedHashMap::empty())
    }

    /// extends a state model by adding additional key/value pairs to the model mapping.
    /// in the case of name collision, we compare old and new state features at that name.
    /// if the state feature has the same unit (tested by StateFeature::Eq), then it can
    /// overwrite the existing.
    ///
    /// this method is used when state models are updated by the user query as Services
    /// become Models in the SearchApp.
    ///
    /// # Arguments
    /// * `query` - JSON search query contents containing state model information
    pub fn extend(
        &self,
        entries: Vec<(String, StateFeature)>,
    ) -> Result<StateModel, StateModelError> {
        let mut map = self
            .0
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<CompactOrderedHashMap<_, _>>();
        let overwrites = entries
            .into_iter()
            .flat_map(|(name, new)| match map.insert(name.clone(), new.clone()) {
                Some(old) if old != new => Some((name.clone(), old, new)),
                _ => None,
            })
            .collect::<Vec<_>>();
        if overwrites.is_empty() {
            Ok(StateModel(map))
        } else {
            let msg = overwrites
                .iter()
                .map(|(k, old, new)| format!("{} old: {} | new: {}", k, old, new))
                .join(", ");
            Err(StateModelError::BuildError(format!(
                "new state features overwriting existing: {}",
                msg
            )))
        }
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn contains_key(&self, k: &String) -> bool {
        self.0.contains_key(k)
    }

    /// collects the state model tuples and clones them so they can
    /// be used to build other collections
    pub fn to_vec(&self) -> Vec<(String, IndexedEntry<StateFeature>)> {
        self.0.to_vec()
    }

    /// iterates over the features in this state in their state vector index ordering.
    pub fn iter(&self) -> FeatureIterator {
        self.0.iter()
    }

    /// iterator that includes the state vector index along with the feature name and StateFeature
    pub fn indexed_iter(&self) -> IndexedFeatureIterator {
        self.0.indexed_iter()
    }

    /// Creates the initial state of a search. this should be a vector of
    /// accumulators, defined in the state model configuration.
    ///
    /// # Returns
    ///
    /// an initialized, "zero"-valued traversal state, or an error
    pub fn initial_state(&self) -> Result<Vec<StateVariable>, StateModelError> {
        self.0
            .iter()
            .map(|(_, feature)| {
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
        state: &[StateVariable],
        name: &String,
        unit: &DistanceUnit,
    ) -> Result<Distance, StateModelError> {
        let value: Distance = self.get_state_variable(state, name)?.into();
        let mut v_cow = Cow::Owned(value);
        let feature = self.get_feature(name)?;
        let from_unit = feature.get_distance_unit()?;

        from_unit.convert(&mut v_cow, unit)?;
        Ok(v_cow.into_owned())
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
        state: &[StateVariable],
        name: &String,
        unit: &TimeUnit,
    ) -> Result<Time, StateModelError> {
        let value: Time = self.get_state_variable(state, name)?.into();
        let mut v_cow = Cow::Owned(value);
        let feature = self.get_feature(name)?;
        let from_unit = feature.get_time_unit()?;

        from_unit.convert(&mut v_cow, unit)?;
        Ok(v_cow.into_owned())
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
        state: &[StateVariable],
        name: &String,
        unit: &EnergyUnit,
    ) -> Result<Energy, StateModelError> {
        let value: Energy = self.get_state_variable(state, name)?.into();
        let mut v_cow = Cow::Owned(value);
        let feature = self.get_feature(name)?;
        let from_unit = feature.get_energy_unit()?;

        from_unit.convert(&mut v_cow, unit)?;
        Ok(v_cow.into_owned())
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
    pub fn get_custom_f64(
        &self,
        state: &[StateVariable],
        name: &String,
    ) -> Result<f64, StateModelError> {
        let (value, format) = self.get_custom_state_variable(state, name)?;
        let result = format.decode_f64(value)?;
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
    pub fn get_custom_i64(
        &self,
        state: &[StateVariable],
        name: &String,
    ) -> Result<i64, StateModelError> {
        let (value, format) = self.get_custom_state_variable(state, name)?;
        let result = format.decode_i64(value)?;
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
    pub fn get_custom_u64(
        &self,
        state: &[StateVariable],
        name: &String,
    ) -> Result<u64, StateModelError> {
        let (value, format) = self.get_custom_state_variable(state, name)?;
        let result = format.decode_u64(value)?;
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
    pub fn get_custom_bool(
        &self,
        state: &[StateVariable],
        name: &String,
    ) -> Result<bool, StateModelError> {
        let (value, format) = self.get_custom_state_variable(state, name)?;
        let result = format.decode_bool(value)?;
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
    fn get_custom_state_variable<'a>(
        &self,
        state: &'a [StateVariable],
        name: &String,
    ) -> Result<(&'a StateVariable, &CustomFeatureFormat), StateModelError> {
        let value = self.get_state_variable(state, name)?;
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
    /// the delta between states for this variable in the state model unit, or an error
    pub fn get_delta<T: From<StateVariable>>(
        &self,
        prev: &[StateVariable],
        next: &[StateVariable],
        name: &String,
    ) -> Result<T, StateModelError> {
        let prev_val = self.get_state_variable(prev, name)?;
        let next_val = self.get_state_variable(next, name)?;
        let delta = *next_val - *prev_val;
        Ok(delta.into())
    }

    /// adds a distance value with distance unit to this feature vector
    pub fn add_distance(
        &self,
        state: &mut [StateVariable],
        name: &String,
        distance: &Distance,
        from_unit: &DistanceUnit,
    ) -> Result<(), StateModelError> {
        let prev_distance = self.get_distance(state, name, from_unit)?;
        let next_distance = prev_distance + *distance;
        self.set_distance(state, name, &next_distance, from_unit)
    }

    /// adds a time value with time unit to this feature vector
    pub fn add_time(
        &self,
        state: &mut [StateVariable],
        name: &String,
        time: &Time,
        from_unit: &TimeUnit,
    ) -> Result<(), StateModelError> {
        let prev_time = self.get_time(state, name, from_unit)?;
        let next_time = prev_time + *time;
        self.set_time(state, name, &next_time, from_unit)
    }

    /// adds a energy value with energy unit to this feature vector
    pub fn add_energy(
        &self,
        state: &mut [StateVariable],
        name: &String,
        energy: &Energy,
        from_unit: &EnergyUnit,
    ) -> Result<(), StateModelError> {
        let prev_energy = self.get_energy(state, name, from_unit)?;
        let next_energy = prev_energy + *energy;
        self.set_energy(state, name, &next_energy, from_unit)
    }

    pub fn set_distance(
        &self,
        state: &mut [StateVariable],
        name: &String,
        distance: &Distance,
        from_unit: &DistanceUnit,
    ) -> Result<(), StateModelError> {
        let mut dist_cow = Cow::Borrowed(distance);
        let to_unit = self.get_feature(name)?.get_distance_unit()?;
        from_unit.convert(&mut dist_cow, to_unit)?;
        self.update_state(
            state,
            name,
            &dist_cow.into_owned().into(),
            UpdateOperation::Replace,
        )
    }

    pub fn set_time(
        &self,
        state: &mut [StateVariable],
        name: &String,
        time: &Time,
        from_unit: &TimeUnit,
    ) -> Result<(), StateModelError> {
        let mut time_mut = Cow::Borrowed(time);
        let to_unit = self.get_feature(name)?.get_time_unit()?;
        from_unit.convert(&mut time_mut, to_unit)?;
        self.update_state(
            state,
            name,
            &time_mut.into_owned().into(),
            UpdateOperation::Replace,
        )
    }

    pub fn set_energy(
        &self,
        state: &mut [StateVariable],
        name: &String,
        energy: &Energy,
        from_unit: &EnergyUnit,
    ) -> Result<(), StateModelError> {
        let mut energy_mut = Cow::Borrowed(energy);
        let to_unit = self.get_feature(name)?.get_energy_unit()?;
        from_unit.convert(&mut energy_mut, to_unit)?;
        self.update_state(
            state,
            name,
            &energy_mut.into_owned().into(),
            UpdateOperation::Replace,
        )
    }

    pub fn set_custom_f64(
        &self,
        state: &mut [StateVariable],
        name: &String,
        value: &f64,
    ) -> Result<(), StateModelError> {
        let feature = self.get_feature(name)?;
        let format = feature.get_custom_feature_format()?;
        let encoded_value = format.encode_f64(value)?;
        self.update_state(state, name, &encoded_value, UpdateOperation::Replace)
    }

    pub fn set_custom_i64(
        &self,
        state: &mut [StateVariable],
        name: &String,
        value: &i64,
    ) -> Result<(), StateModelError> {
        let feature = self.get_feature(name)?;
        let format = feature.get_custom_feature_format()?;
        let encoded_value = format.encode_i64(value)?;
        self.update_state(state, name, &encoded_value, UpdateOperation::Replace)
    }

    pub fn set_custom_u64(
        &self,
        state: &mut [StateVariable],
        name: &String,
        value: &u64,
    ) -> Result<(), StateModelError> {
        let feature = self.get_feature(name)?;
        let format = feature.get_custom_feature_format()?;
        let encoded_value = format.encode_u64(value)?;
        self.update_state(state, name, &encoded_value, UpdateOperation::Replace)
    }

    pub fn set_custom_bool(
        &self,
        state: &mut [StateVariable],
        name: &String,
        value: &bool,
    ) -> Result<(), StateModelError> {
        let feature = self.get_feature(name)?;
        let format = feature.get_custom_feature_format()?;
        let encoded_value = format.encode_bool(value)?;
        self.update_state(state, name, &encoded_value, UpdateOperation::Replace)
    }

    /// uses the state model to pretty print a state instance as a JSON object
    ///
    /// # Arguments
    /// * `state` - any (valid) state vector instance
    ///
    /// # Result
    /// A JSON object representation of that vector
    pub fn serialize_state(&self, state: &[StateVariable]) -> serde_json::Value {
        let output = self
            .iter()
            .zip(state.iter())
            .map(|((name, _), state_var)| (name, state_var))
            .collect::<HashMap<_, _>>();
        json![output]
    }

    /// uses the built-in serialization codec to output the state model representation as a JSON object
    /// stores the result as a JSON Object (Map).
    pub fn serialize_state_model(&self) -> serde_json::Value {
        let mut out = serde_json::Map::new();
        for (i, (name, feature)) in self.indexed_iter() {
            let mut f_json = json![feature];

            if let Some(map) = f_json.as_object_mut() {
                map.insert(String::from("index"), json![i]);
                map.insert(String::from("name"), json![name]);
            }
            out.insert(name.clone(), f_json);
        }

        json![out]
    }

    /// lists the names of the state variables in order
    pub fn get_names(&self) -> String {
        self.0.iter().map(|(k, _)| k.clone()).join(",")
    }

    fn get_feature(&self, feature_name: &String) -> Result<&StateFeature, StateModelError> {
        self.0.get(feature_name).ok_or_else(|| {
            StateModelError::UnknownStateVariableName(feature_name.clone(), self.get_names())
        })
    }

    /// gets a state variable from a state vector by name
    fn get_state_variable<'a>(
        &self,
        state: &'a [StateVariable],
        name: &String,
    ) -> Result<&'a StateVariable, StateModelError> {
        let idx = self.0.get_index(name).ok_or_else(|| {
            StateModelError::UnknownStateVariableName(name.clone(), self.get_names())
        })?;
        let value = state.get(idx).ok_or_else(|| {
            StateModelError::RuntimeError(format!(
                "state index {} for {} is out of range for state vector with {} entries",
                idx,
                name,
                state.len()
            ))
        })?;
        Ok(value)
    }

    fn update_state(
        &self,
        state: &mut [StateVariable],
        name: &String,
        value: &StateVariable,
        op: UpdateOperation,
    ) -> Result<(), StateModelError> {
        let index = self.0.get_index(name).ok_or_else(|| {
            StateModelError::UnknownStateVariableName(name.clone(), self.get_names())
        })?;
        let prev = state
            .get(index)
            .ok_or(StateModelError::InvalidStateVariableIndex(
                index,
                state.len(),
            ))?;
        let updated = op.perform_operation(prev, value);
        state[index] = updated;
        Ok(())
    }
}

impl<'a> TryFrom<&'a serde_json::Value> for StateModel {
    type Error = StateModelError;

    /// builds a new state model from a JSON array of deserialized StateFeatures.
    /// the size of the JSON object matches the size of the feature vector. downstream
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
    /// [state]
    /// distance = { "distance_unit" = "kilometers", initial = 0.0 },
    /// time = { "time_unit" = "minutes", initial = 0.0 },
    /// battery_soc = { name = "soc", unit = "percent", format = { type = "floating_point", initial = 0.0 } }
    ///
    /// the same example as JSON (convert '=' into ':', and enquote object keys):
    ///
    /// ```json
    /// {
    ///   "distance": { "distance_unit": "kilometers", "initial": 0.0 },
    ///   "time": { "time_unit": "minutes", "initial": 0.0 },
    ///   "battery_soc": {
    ///     "name": "soc",
    ///     "unit": "percent",
    ///     "format": {
    ///       "type": "floating_point",
    ///       "initial": 0.0
    ///     }
    ///   }
    /// }
    /// ```
    fn try_from(json: &'a serde_json::Value) -> Result<StateModel, StateModelError> {
        let tuples = json
            .as_object()
            .ok_or_else(|| {
                StateModelError::BuildError(String::from(
                    "expected state model configuration to be a JSON object {}",
                ))
            })?
            .into_iter()
            .map(|(feature_name, feature_json)| {
                let feature = serde_json::from_value::<StateFeature>(feature_json.clone())
                    .map_err(|e| {
                        StateModelError::BuildError(format!(
                        "unable to parse state feature row with name '{}' contents '{}' due to: {}",
                        feature_name.clone(),
                        feature_json.clone(),
                        e
                    ))
                    })?;
                Ok((feature_name.clone(), feature))
            })
            .collect::<Result<Vec<_>, StateModelError>>()?;
        let state_model = StateModel::from(tuples);
        Ok(state_model)
    }
}

impl From<Vec<(String, StateFeature)>> for StateModel {
    fn from(value: Vec<(String, StateFeature)>) -> Self {
        StateModel::new(value)
    }
}
