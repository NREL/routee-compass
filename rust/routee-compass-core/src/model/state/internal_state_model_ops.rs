use super::{
    state_error::StateError, state_feature::StateFeature, state_model::StateModel,
    update_operation::UpdateOperation,
};
use crate::model::traversal::state::state_variable::StateVar;

/// this trait provides the internal API for methods on a state model.
/// it is not intended for use by the downstream models (TraversalModel, CostModel, FrontierModel);
/// for those, stick to the methods defined on a StateModel.
pub(crate) trait InternalStateModelOps {
    /// get the names of features for this state model
    fn get_names(&self) -> Vec<String>;
    /// get the names of features as a comma-delimited string
    fn get_namelist(&self) -> String;
    /// internal API that gets a feature name / state feature pair by index
    fn get(&self, index: usize) -> Result<(&String, &StateFeature), StateError>;
    /// get the index of a feature by feature name
    fn get_index(&self, name: &str) -> Result<usize, StateError>;
    /// get the value of a feature from a state vector by feature name.
    /// use get methods defined on StateModel instead.
    fn get_value(&self, state: &[StateVar], name: &str) -> Result<StateVar, StateError>;
    /// get the state feature object for some feature name.
    fn get_feature(&self, name: &str) -> Result<&StateFeature, StateError>;
    /// updates a feature in a state vector for some feature name by taking the new
    /// value and applying an update operation during assignment.
    fn update_state(
        &self,
        state: &mut [StateVar],
        name: &str,
        value: &StateVar,
        op: UpdateOperation,
    ) -> Result<(), StateError>;
}

impl InternalStateModelOps for StateModel {
    fn get_names(&self) -> Vec<String> {
        match self {
            StateModel::OneFeature { key, .. } => vec![key.clone()],
            StateModel::TwoFeatures { k1, k2, .. } => vec![k1.clone(), k2.clone()],
            StateModel::ThreeFeatures { k1, k2, k3, .. } => {
                vec![k1.clone(), k2.clone(), k3.clone()]
            }
            StateModel::FourFeatures { k1, k2, k3, k4, .. } => {
                vec![k1.clone(), k2.clone(), k3.clone(), k4.clone()]
            }
            StateModel::NFeatures(f) => f.keys().cloned().collect(),
        }
    }

    fn get_namelist(&self) -> String {
        let names = self.get_names().join(", ");
        format!("[{}]", names)
    }

    fn get(&self, index: usize) -> Result<(&String, &StateFeature), StateError> {
        let kv_option = match self {
            StateModel::OneFeature { key, value } => {
                if index == 0 {
                    Some((key, value))
                } else {
                    None
                }
            }
            StateModel::TwoFeatures { k1, k2, v1, v2 } => {
                if index == 0 {
                    Some((k1, v1))
                } else if index == 1 {
                    Some((k2, v2))
                } else {
                    None
                }
            }
            StateModel::ThreeFeatures {
                k1,
                k2,
                k3,
                v1,
                v2,
                v3,
            } => {
                if index == 0 {
                    Some((k1, v1))
                } else if index == 1 {
                    Some((k2, v2))
                } else if index == 2 {
                    Some((k3, v3))
                } else {
                    None
                }
            }
            StateModel::FourFeatures {
                k1,
                k2,
                k3,
                k4,
                v1,
                v2,
                v3,
                v4,
            } => {
                if index == 0 {
                    Some((k1, v1))
                } else if index == 1 {
                    Some((k2, v2))
                } else if index == 2 {
                    Some((k3, v3))
                } else if index == 3 {
                    Some((k4, v4))
                } else {
                    None
                }
            }
            StateModel::NFeatures(indexed) => {
                if index > indexed.len() {
                    None
                } else {
                    indexed
                        .iter()
                        .find(|(_, f)| f.index == index)
                        .map(|(n, isf)| (n, &isf.feature))
                }
            }
        };
        kv_option.ok_or_else(|| StateError::InvalidStateVariableIndex(index, self.len()))
    }

    fn get_index(&self, name: &str) -> Result<usize, StateError> {
        let idx_option: Option<usize> = match self {
            StateModel::OneFeature { key, .. } => {
                if name == key {
                    Some(0)
                } else {
                    None
                }
            }
            StateModel::TwoFeatures { k1, k2, .. } => {
                if name == k1 {
                    Some(0)
                } else if name == k2 {
                    Some(1)
                } else {
                    None
                }
            }
            StateModel::ThreeFeatures { k1, k2, k3, .. } => {
                if name == k1 {
                    Some(0)
                } else if name == k2 {
                    Some(1)
                } else if name == k3 {
                    Some(2)
                } else {
                    None
                }
            }
            StateModel::FourFeatures { k1, k2, k3, k4, .. } => {
                if name == k1 {
                    Some(0)
                } else if name == k2 {
                    Some(1)
                } else if name == k3 {
                    Some(2)
                } else if name == k4 {
                    Some(3)
                } else {
                    None
                }
            }
            StateModel::NFeatures(indexed) => indexed.get(name).map(|f| f.index),
        };
        idx_option.ok_or_else(|| {
            let names = self.get_namelist();
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

    fn get_feature(&self, name: &str) -> Result<&StateFeature, StateError> {
        let feature_option = match self {
            StateModel::OneFeature { key, value } => {
                if name == key {
                    Some(value)
                } else {
                    None
                }
            }
            StateModel::TwoFeatures { k1, k2, v1, v2 } => {
                if name == k1 {
                    Some(v1)
                } else if name == k2 {
                    Some(v2)
                } else {
                    None
                }
            }
            StateModel::ThreeFeatures {
                k1,
                k2,
                k3,
                v1,
                v2,
                v3,
            } => {
                if name == k1 {
                    Some(v1)
                } else if name == k2 {
                    Some(v2)
                } else if name == k3 {
                    Some(v3)
                } else {
                    None
                }
            }
            StateModel::FourFeatures {
                k1,
                k2,
                k3,
                k4,
                v1,
                v2,
                v3,
                v4,
            } => {
                if name == k1 {
                    Some(v1)
                } else if name == k2 {
                    Some(v2)
                } else if name == k3 {
                    Some(v3)
                } else if name == k4 {
                    Some(v4)
                } else {
                    None
                }
            }
            StateModel::NFeatures(indexed) => indexed.get(name).map(|f| &f.feature),
        };
        feature_option.ok_or_else(|| {
            let names = self.get_namelist();
            StateError::UnknownStateVariableName(name.into(), names)
        })
    }

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
}
