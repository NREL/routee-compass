use super::state_feature::StateFeature;

/// simple record type which couples the state variable index with the feature
/// representation for a given state variable in the StateModel.
pub struct StateModelEntry {
    pub index: usize,
    pub feature: StateFeature,
}
