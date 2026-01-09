use crate::model::traversal::{TraversalModel, TraversalModelError, TraversalModelService};
use itertools::Itertools;
use std::sync::Arc;
use topological_sort;

/// sorts the traversal models based on service feature declarations such that the inter-model
/// dependencies are sorted. in other words: a time model depends on a distance calculation <->
/// the distance model must appear earlier in the list than the time model.
///
/// only confirms that the names match, ignores confirming the feature types match.
///
/// # Arguments
///
/// * `services` - the traversal model services
/// * `models` - the corresponding traversal models built from those services
///
/// # Returns
///
/// Sorted list of models, or an error if dependencies are missing
pub fn topological_dependency_sort_services(
    services: &[Arc<dyn TraversalModelService>],
    models: &[Arc<dyn TraversalModel>],
) -> Result<Vec<Arc<dyn TraversalModel>>, TraversalModelError> {
    if services.len() != models.len() {
        return Err(TraversalModelError::BuildError(format!(
            "services and models must have same length: {} vs {}",
            services.len(),
            models.len()
        )));
    }

    let output_features_lookup = services
        .iter()
        .enumerate()
        .flat_map(|(idx, s)| s.output_features().into_iter().map(move |(n, _)| (n, idx)))
        .into_group_map();

    // find relationships between model input and output features
    let mut missing_parents: Vec<String> = vec![];
    let mut sort = topological_sort::TopologicalSort::<usize>::new();
    for (idx, s) in services.iter().enumerate() {
        let input_features = s.input_features();
        if input_features.is_empty() {
            sort.insert(idx);
        } else {
            for feature in input_features.iter() {
                match &output_features_lookup.get(&feature.name()) {
                    None => {
                        missing_parents.push(feature.name());
                    }
                    Some(ref_idxs) => {
                        // look at all dependencies but ignore self-looping dependencies
                        let refs_iter = ref_idxs.iter().filter(|i| &idx != *i);
                        for ref_idx in refs_iter {
                            sort.add_dependency(idx, *ref_idx);
                        }
                    }
                }
            }
        }
    }

    if !missing_parents.is_empty() {
        let joined = missing_parents.iter().join(",");
        let msg = format!(
            "the following state variables are required by traversal models but missing: {{{joined}}}"
        );
        return Err(TraversalModelError::BuildError(msg));
    }

    // apply topological sort to the models.
    // the correct order we want is the opposite of the popped result
    let mut result = vec![];
    while let Some(m_idx) = sort.pop() {
        let model = models.get(m_idx).ok_or_else(|| {
            TraversalModelError::BuildError(format!("internal error: sort has model index {m_idx} which is not found in the model collection"))
        })?;
        result.push(model.clone());
    }
    result.reverse();

    log::debug!(
        "topological sort of traversal models: {}",
        result.iter().map(|m| m.name()).join(", ")
    );

    // topological_sort crate's pop() method stops when collection is empty, unless there is a cyclical
    // dependency, in which case the collection will return None and remain non-empty.
    if !sort.is_empty() {
        let msg = format!("cyclical dependency in traversal model features: {sort:?}");
        return Err(TraversalModelError::BuildError(msg));
    }

    Ok(result)
}

/// sorts the traversal models such that the inter-model dependencies are sorted.
/// in other words: a time model depends on a distance calculation <-> the distance model
/// must appear earlier in the list than the time model.
///
/// only confirms that the names match, ignores confirming the feature types match.
///
/// # Arguments
///
/// * `models` - the traversal models to sort
///
/// # Returns
///
/// Sorted list of models, or an error if dependencies are missing
///
/// # Deprecated
///
/// This function is deprecated. Use `topological_dependency_sort_services` instead.
#[deprecated(
    since = "0.1.0",
    note = "Use topological_dependency_sort_services instead, which takes services to access feature information"
)]
pub fn topological_dependency_sort(
    _models: &[Arc<dyn TraversalModel>],
) -> Result<Vec<Arc<dyn TraversalModel>>, TraversalModelError> {
    panic!("topological_dependency_sort is deprecated. Use topological_dependency_sort_services instead, which requires both services and models.");
}

#[cfg(test)]
mod test {

    use super::{topological_dependency_sort, topological_dependency_sort_services};
    use crate::{
        algorithm::search::SearchTree,
        model::{
            network::{Edge, Vertex},
            state::{InputFeature, StateModel, StateVariable, StateVariableConfig},
            traversal::{TraversalModel, TraversalModelError, TraversalModelService},
        },
    };
    use itertools::Itertools;
    use std::sync::Arc;
    use uom::{si::f64::Length, ConstZero};

    /// tests dependency sort on the typical setup for modeling distance, speed,
    /// time, grade, elevation, and energy.
    #[test]
    fn test_default_model_setup() {
        init_test_logger();

        let distance_feature = InputFeature::Distance {
            name: "distance".to_string(),
            unit: None,
        };
        let speed_feature = InputFeature::Speed {
            name: "speed".to_string(),
            unit: None,
        };
        let grade_feature = InputFeature::Ratio {
            name: "grade".to_string(),
            unit: None,
        };

        // mock up services for 6 dimensions, 3 of which have upstream dependencies
        let distance_svc = MockModelService::new(vec![], vec!["distance"]);
        let speed_svc = MockModelService::new(vec![], vec!["speed"]);
        let time_svc = MockModelService::new(
            vec![distance_feature.clone(), speed_feature.clone()],
            vec!["time"],
        );
        let grade_svc = MockModelService::new(vec![], vec!["grade"]);
        let elevation_svc = MockModelService::new(vec![grade_feature.clone()], vec!["elevation"]);
        let energy_svc = MockModelService::new(
            vec![distance_feature, speed_feature, grade_feature],
            vec!["energy"],
        );
        let services: Vec<Arc<dyn TraversalModelService>> =
            vec![energy_svc, elevation_svc, time_svc, grade_svc, speed_svc, distance_svc]
                .into_iter()
                .map(|s| {
                    let svc: Arc<dyn TraversalModelService> = Arc::new(s);
                    svc
                })
                .collect_vec();
        
        // Build models from services
        let models: Vec<Arc<dyn TraversalModel>> = services
            .iter()
            .map(|svc| svc.build(&serde_json::Value::Null).expect("build failed"))
            .collect_vec();

        // apply sort and then reconstruct descriptions for each model on the sorted values
        let sorted = topological_dependency_sort_services(&services, &models).expect("failure during sort function");
        let sorted_descriptions = sorted
            .iter()
            .map(|m| {
                // Find the matching service for this model by matching names
                let model_name = m.name();
                let svc = services.iter()
                    .find(|s| {
                        // Build a model from the service to check its name
                        s.build(&serde_json::Value::Null)
                            .map(|built| built.name() == model_name)
                            .unwrap_or(false)
                    })
                    .expect("should find matching service");
                
                let input_features = svc.input_features();
                let in_names = if input_features.is_empty() {
                    String::from("*")
                } else {
                    svc.input_features()
                        .iter()
                        .map(|feature| feature.name())
                        .join("+")
                };
                let out_name = svc.output_features().iter().map(|(n, _)| n).join("");
                format!("{in_names}->{out_name}")
            })
            .collect_vec();

        // validate that the dependencies are honored. the algorithm makes no guarantees about
        // tiebreaking usize values so we cannot expect a deterministic output here.
        let mut distance_seen = false;
        let mut speed_seen = false;
        let mut grade_seen = false;
        for description in sorted_descriptions.iter() {
            match description.as_str() {
                "*->grade" => grade_seen = true,
                "*->speed" => speed_seen = true,
                "*->distance" => distance_seen = true,
                "distance+speed+grade->energy" => {
                    assert!(distance_seen && speed_seen && grade_seen)
                }
                "distance+speed->time" => assert!(distance_seen && speed_seen),
                "grade->elevation" => assert!(grade_seen),
                other => panic!("unexpected key: '{other}'"),
            }
        }
    }

    /// tests dependency sort where the energy model depends on and generates SOC values
    #[test]
    fn test_self_dependency() {
        init_test_logger();

        let distance_feature = InputFeature::Distance {
            name: "distance".to_string(),
            unit: None,
        };
        let speed_feature = InputFeature::Speed {
            name: "speed".to_string(),
            unit: None,
        };
        let grade_feature = InputFeature::Ratio {
            name: "grade".to_string(),
            unit: None,
        };
        let soc_feature = InputFeature::Ratio {
            name: "soc".to_string(),
            unit: None,
        };

        // mock up services for 6 dimensions, 3 of which have upstream dependencies
        let distance_svc = MockModelService::new(vec![], vec!["distance"]);
        let speed_svc = MockModelService::new(vec![], vec!["speed"]);
        let time_svc = MockModelService::new(
            vec![distance_feature.clone(), speed_feature.clone()],
            vec!["time"],
        );
        let grade_svc = MockModelService::new(vec![], vec!["grade"]);
        let elevation_svc = MockModelService::new(vec![grade_feature.clone()], vec!["elevation"]);
        let energy_svc = MockModelService::new(
            vec![distance_feature, speed_feature, grade_feature, soc_feature],
            vec!["energy", "soc"],
        );
        let services: Vec<Arc<dyn TraversalModelService>> =
            vec![energy_svc, elevation_svc, time_svc, grade_svc, speed_svc, distance_svc]
                .into_iter()
                .map(|s| {
                    let svc: Arc<dyn TraversalModelService> = Arc::new(s);
                    svc
                })
                .collect_vec();

        // Build models from services
        let models: Vec<Arc<dyn TraversalModel>> = services
            .iter()
            .map(|svc| svc.build(&serde_json::Value::Null).expect("build failed"))
            .collect_vec();

        // apply sort and then reconstruct descriptions for each model on the sorted values
        let sorted = topological_dependency_sort_services(&services, &models).expect("failure during sort function");
        let sorted_descriptions = sorted
            .iter()
            .map(|m| {
                // Find the matching service for this model by matching names
                let model_name = m.name();
                let svc = services.iter()
                    .find(|s| {
                        // Build a model from the service to check its name
                        s.build(&serde_json::Value::Null)
                            .map(|built| built.name() == model_name)
                            .unwrap_or(false)
                    })
                    .expect("should find matching service");
                
                let input_features = svc.input_features();
                let in_names = if input_features.is_empty() {
                    String::from("*")
                } else {
                    input_features.iter().map(|f| f.name()).join("+")
                };
                let out_name = svc.output_features().iter().map(|(n, _)| n).join("+");
                format!("{in_names}->{out_name}")
            })
            .collect_vec();

        // validate that the dependencies are honored. the algorithm makes no guarantees about
        // tiebreaking usize values so we cannot expect a deterministic output here.
        let mut distance_seen = false;
        let mut speed_seen = false;
        let mut grade_seen = false;
        for description in sorted_descriptions.iter() {
            match description.as_str() {
                "*->grade" => grade_seen = true,
                "*->speed" => speed_seen = true,
                "*->distance" => distance_seen = true,
                "distance+speed+grade+soc->energy+soc" => {
                    assert!(distance_seen && speed_seen && grade_seen)
                }
                "distance+speed->time" => assert!(distance_seen && speed_seen),
                "grade->elevation" => assert!(grade_seen),
                other => panic!("unexpected key: '{other}'"),
            }
        }
    }

    fn init_test_logger() {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Debug)
            .try_init();
    }

    struct MockModel {
        in_features: Vec<InputFeature>,
        out_features: Vec<String>,
    }

    impl MockModel {
        pub fn new(in_features: Vec<InputFeature>, out_features: Vec<&str>) -> MockModel {
            MockModel {
                in_features,
                out_features: out_features.into_iter().map(String::from).collect_vec(),
            }
        }
    }

    struct MockModelService {
        in_features: Vec<InputFeature>,
        out_features: Vec<String>,
    }

    impl MockModelService {
        pub fn new(in_features: Vec<InputFeature>, out_features: Vec<&str>) -> MockModelService {
            MockModelService {
                in_features,
                out_features: out_features.into_iter().map(String::from).collect_vec(),
            }
        }
    }

    impl TraversalModelService for MockModelService {
        fn build(
            &self,
            _parameters: &serde_json::Value,
        ) -> Result<Arc<dyn TraversalModel>, TraversalModelError> {
            Ok(Arc::new(MockModel {
                in_features: self.in_features.clone(),
                out_features: self.out_features.clone(),
            }))
        }

        fn input_features(&self) -> Vec<InputFeature> {
            self.in_features.clone()
        }

        fn output_features(&self) -> Vec<(String, StateVariableConfig)> {
            self.out_features
                .iter()
                .map(|n| {
                    (
                        n.clone(),
                        StateVariableConfig::Distance {
                            initial: Length::ZERO,
                            accumulator: true,
                            output_unit: None,
                        },
                    )
                })
                .collect_vec()
        }
    }

    impl TraversalModel for MockModel {
        fn name(&self) -> String {
            format!(
                "Mock Traversal Model: {} -> {}",
                self.in_features.iter().map(|f| f.name()).join("+"),
                self.out_features.join("+")
            )
        }

        fn traverse_edge(
            &self,
            _trajectory: (&Vertex, &Edge, &Vertex),
            _state: &mut Vec<StateVariable>,
            _tree: &SearchTree,
            _state_model: &StateModel,
        ) -> Result<(), TraversalModelError> {
            todo!()
        }

        fn estimate_traversal(
            &self,
            _od: (&Vertex, &Vertex),
            _state: &mut Vec<StateVariable>,
            _tree: &SearchTree,
            _state_model: &StateModel,
        ) -> Result<(), TraversalModelError> {
            todo!()
        }
    }
}
