use itertools::Itertools;
use routee_compass_core::config::ConfigJsonExtensions;
use routee_compass_core::model::{
    access::AccessModel,
    state::{StateFeature, StateModelError},
    traversal::TraversalModel,
};
use std::{collections::HashMap, sync::Arc};

/// collects the state features to use in this search. the features are collected in
/// the following order:
///   1. from the traversal model
///   2. from the access model
///   3. optionally from the query itself
///
/// using the order above, each new source optionally overwrites any existing feature
/// by name (tuple index 0) as long as they match in StateFeature::get_feature_name and
/// StateFeature::get_feature_unit_name.
pub fn collect_features(
    query: &serde_json::Value,
    traversal_model: Arc<dyn TraversalModel>,
    access_model: Arc<dyn AccessModel>,
) -> Result<Vec<(String, StateFeature)>, StateModelError> {
    // prepare the set of features for this state model
    let model_features = traversal_model
        .output_features()
        .into_iter()
        .chain(access_model.state_features())
        .collect::<HashMap<_, _>>();

    // build the state model. inject state features from the traversal and access models
    // and then allow the user to optionally override any initial conditions for those
    // state features.
    let user_features_option: Option<HashMap<String, StateFeature>> = query
        .get_config_serde_optional(&"state_features", &"query")
        .map_err(|e| StateModelError::BuildError(e.to_string()))?;
    let user_features = user_features_option
        .unwrap_or_default()
        .into_iter()
        .map(|(name, feature)| match model_features.get(&name) {
            None => {
                let fnames = model_features.keys().join(",");
                Err(StateModelError::UnknownStateVariableName(name, fnames))
            }
            Some(existing) if existing.get_feature_type() != feature.get_feature_type() => {
                Err(StateModelError::UnexpectedFeatureType(
                    existing.get_feature_type(),
                    feature.get_feature_type(),
                ))
            }
            Some(_) => Ok((name, feature)),
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut added_features: Vec<(String, StateFeature)> = model_features.into_iter().collect_vec();
    added_features.extend(user_features);
    Ok(added_features)
}
