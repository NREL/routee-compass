use uom::ConstZero;

use crate::algorithm::search::SearchTree;
use crate::model::network::{Edge, Vertex};
use crate::model::state::{CustomVariableConfig, InputFeature};
use crate::model::traversal::{TraversalModel, TraversalModelService};
use crate::model::{
    state::StateVariableConfig,
    traversal::{default::combined::CombinedTraversalModel, TraversalModelError},
};
use std::sync::Arc;

/// a traversal model that can be used in tests which will "plug in" a mock model that
/// registers all of the input feature requirements for the provided model.
pub struct TestTraversalModel {}

pub struct TestTraversalModelResult {
    pub model: Arc<dyn TraversalModel>,
    pub input_features: Vec<InputFeature>,
    pub output_features: Vec<(String, StateVariableConfig)>,
}

impl TestTraversalModel {
    /// build (wrap) the model for testing.
    /// Takes a service to access feature requirements, and builds a model from it.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        service: Arc<dyn TraversalModelService>,
    ) -> Result<TestTraversalModelResult, TraversalModelError> {
        let model = service.build(&serde_json::json!({}))?;
        let upstream: Box<dyn TraversalModel> =
            Box::new(MockUpstreamModel::new_upstream_from(service.clone()));
        let wrapped: Arc<dyn TraversalModel> = Arc::new(CombinedTraversalModel::new(vec![
            Arc::from(upstream),
            model.clone(),
        ]));
        
        // Collect all input features from the service and convert them to output features for mocking
        let mut output_features: Vec<(String, StateVariableConfig)> = Vec::new();
        for input_feature in service.input_features().iter() {
            let output = MockUpstreamModel::input_feature_to_output_config(input_feature);
            output_features.push(output);
        }
        // Add the actual output features from the service
        output_features.extend(service.output_features());
        
        Ok(TestTraversalModelResult {
            model: wrapped,
            input_features: vec![],
            output_features,
        })
    }
}

pub struct MockUpstreamModel {
    input_features: Vec<InputFeature>,
    output_features: Vec<(String, StateVariableConfig)>,
}

impl MockUpstreamModel {
    /// builds a new mock upstream TraversalModel that registers all of the input feature
    /// requirements for the provided service.
    pub fn new_upstream_from(service: Arc<dyn TraversalModelService>) -> MockUpstreamModel {
        let input_features = vec![];
        let output_features = service
            .input_features()
            .iter()
            .map(|feature| match feature {
                InputFeature::Distance { name, unit: _ } => (
                    name.clone(),
                    StateVariableConfig::Distance {
                        initial: uom::si::f64::Length::ZERO,
                        accumulator: false,
                        output_unit: None,
                    },
                ),
                InputFeature::Ratio { name, unit: _ } => (
                    name.clone(),
                    StateVariableConfig::Ratio {
                        initial: uom::si::f64::Ratio::ZERO,
                        accumulator: false,
                        output_unit: None,
                    },
                ),
                InputFeature::Speed { name, unit: _ } => (
                    name.clone(),
                    StateVariableConfig::Speed {
                        initial: uom::si::f64::Velocity::ZERO,
                        accumulator: false,
                        output_unit: None,
                    },
                ),
                InputFeature::Time { name, unit: _ } => (
                    name.clone(),
                    StateVariableConfig::Time {
                        initial: uom::si::f64::Time::ZERO,
                        accumulator: false,
                        output_unit: None,
                    },
                ),
                InputFeature::Energy { name, unit: _ } => (
                    name.clone(),
                    StateVariableConfig::Energy {
                        initial: uom::si::f64::Energy::ZERO,
                        accumulator: false,
                        output_unit: None,
                    },
                ),
                InputFeature::Temperature { name, unit: _ } => (
                    name.clone(),
                    StateVariableConfig::Temperature {
                        initial: uom::si::f64::ThermodynamicTemperature::ZERO,
                        accumulator: false,
                        output_unit: None,
                    },
                ),
                InputFeature::Custom { name, unit } => {
                    // only current way to hook in custom unit type
                    use CustomVariableConfig as C;
                    let var_config = match unit.as_str() {
                        "floating_point" => C::FloatingPoint {
                            initial: ordered_float::OrderedFloat(0.0),
                        },
                        "signed_integer" => C::SignedInteger { initial: 0 },
                        "unsigned_integer" => C::UnsignedInteger { initial: 0 },
                        "boolean" => C::Boolean { initial: false },
                        _ => C::FloatingPoint {
                            initial: ordered_float::OrderedFloat(0.0),
                        },
                    };
                    (
                        name.clone(),
                        StateVariableConfig::Custom {
                            custom_type: name.clone(),
                            accumulator: false,
                            value: var_config,
                        },
                    )
                }
            })
            .collect();
        Self {
            input_features,
            output_features,
        }
    }
    
    /// Helper function to convert an InputFeature to a StateVariableConfig for mocking
    fn input_feature_to_output_config(feature: &InputFeature) -> (String, StateVariableConfig) {
        match feature {
            InputFeature::Distance { name, .. } => (
                name.clone(),
                StateVariableConfig::Distance {
                    initial: uom::si::f64::Length::ZERO,
                    accumulator: false,
                    output_unit: None,
                },
            ),
            InputFeature::Ratio { name, .. } => (
                name.clone(),
                StateVariableConfig::Ratio {
                    initial: uom::si::f64::Ratio::ZERO,
                    accumulator: false,
                    output_unit: None,
                },
            ),
            InputFeature::Speed { name, .. } => (
                name.clone(),
                StateVariableConfig::Speed {
                    initial: uom::si::f64::Velocity::ZERO,
                    accumulator: false,
                    output_unit: None,
                },
            ),
            InputFeature::Time { name, .. } => (
                name.clone(),
                StateVariableConfig::Time {
                    initial: uom::si::f64::Time::ZERO,
                    accumulator: false,
                    output_unit: None,
                },
            ),
            InputFeature::Energy { name, .. } => (
                name.clone(),
                StateVariableConfig::Energy {
                    initial: uom::si::f64::Energy::ZERO,
                    accumulator: false,
                    output_unit: None,
                },
            ),
            InputFeature::Temperature { name, .. } => (
                name.clone(),
                StateVariableConfig::Temperature {
                    initial: uom::si::f64::ThermodynamicTemperature::ZERO,
                    accumulator: false,
                    output_unit: None,
                },
            ),
            InputFeature::Custom { name, .. } => (
                name.clone(),
                StateVariableConfig::Custom {
                    custom_type: name.clone(),
                    value: CustomVariableConfig::FloatingPoint {
                        initial: ordered_float::OrderedFloat(0.0),
                    },
                    accumulator: false,
                },
            ),
        }
    }
}

impl TraversalModel for MockUpstreamModel {
    fn name(&self) -> String {
        String::from("Mock Upstream Traversal Model")
    }

    fn traverse_edge(
        &self,
        _trajectory: (&Vertex, &Edge, &Vertex),
        _state: &mut Vec<crate::model::state::StateVariable>,
        _tree: &SearchTree,
        _state_model: &crate::model::state::StateModel,
    ) -> Result<(), TraversalModelError> {
        Ok(())
    }

    fn estimate_traversal(
        &self,
        _od: (&Vertex, &Vertex),
        _state: &mut Vec<crate::model::state::StateVariable>,
        _tree: &SearchTree,
        _state_model: &crate::model::state::StateModel,
    ) -> Result<(), TraversalModelError> {
        Ok(())
    }
}
