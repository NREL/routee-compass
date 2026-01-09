use uom::ConstZero;

use crate::algorithm::search::SearchTree;
use crate::model::network::{Edge, Vertex};
use crate::model::state::{CustomVariableConfig, InputFeature, StateModel};
use crate::model::traversal::{TraversalModel, TraversalModelService};
use crate::model::{
    state::StateVariableConfig,
    traversal::{default::combined::CombinedTraversalModel, TraversalModelError},
};
use std::sync::Arc;

/// A test helper that wraps a `TraversalModelService` with a mock upstream service
/// that provides all required input features. This allows testing traversal models
/// in isolation without needing to set up their dependencies.
pub struct TestTraversalModel {}

/// The result of wrapping a service for testing, containing the combined service
/// and its feature specifications.
pub struct TestTraversalModelResult {
    /// A pre-built model for convenience in simple test cases
    pub model: Arc<dyn TraversalModel>,
    /// The combined service that includes both the mock upstream and the service under test
    pub service: Arc<dyn TraversalModelService>,
    /// Empty vector since all inputs are satisfied by the mock upstream
    pub input_features: Vec<InputFeature>,
    /// All output features from both the mock upstream and the service under test
    pub output_features: Vec<(String, StateVariableConfig)>,
}

impl TestTraversalModelResult {
    /// Builds a `TraversalModel` from the wrapped service for a given query.
    ///
    /// This is a convenience method that:
    /// 1. Creates a `StateModel` with the appropriate feature registrations
    /// 2. Builds a `TraversalModel` from the service
    ///
    /// # Arguments
    ///
    /// * `query` - The query parameters for building the model (can be empty JSON object)
    ///
    /// # Returns
    ///
    /// A tuple of (model, state_model) ready for testing
    pub fn build_model(
        &self,
        query: &serde_json::Value,
    ) -> Result<(Arc<dyn TraversalModel>, Arc<StateModel>), TraversalModelError> {
        let state_model = Arc::new(
            StateModel::empty()
                .register(self.input_features.clone(), self.output_features.clone())?,
        );
        let model = self.service.build(query, state_model.clone())?;
        Ok((model, state_model))
    }

    /// Convenience method to build a model with an empty query.
    pub fn build_model_default(
        &self,
    ) -> Result<(Arc<dyn TraversalModel>, Arc<StateModel>), TraversalModelError> {
        self.build_model(&serde_json::json!({}))
    }
}

impl TestTraversalModel {
    /// Wraps a `TraversalModelService` with a mock upstream service for testing.
    ///
    /// The mock upstream service will provide all input features required by the
    /// service under test, allowing it to be tested in isolation.
    ///
    /// # Arguments
    ///
    /// * `service` - The service to wrap for testing
    ///
    /// # Returns
    ///
    /// A `TestTraversalModelResult` containing the wrapped service and its features
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        service: Arc<dyn TraversalModelService>,
    ) -> Result<TestTraversalModelResult, TraversalModelError> {
        // Create mock upstream service that provides all required inputs
        let mock_service = Arc::new(MockUpstreamService::new_from(service.clone()));

        // Combine mock and actual services
        let combined_service = Arc::new(CombinedTraversalService::new(vec![
            mock_service.clone(),
            service.clone(),
        ]));

        // Collect all output features from both services
        let mut all_output_features: Vec<(String, StateVariableConfig)> = Vec::new();
        all_output_features.extend(mock_service.output_features());
        all_output_features.extend(service.output_features());

        // Build a default model for convenience
        let state_model =
            Arc::new(StateModel::empty().register(vec![], all_output_features.clone())?);
        let model = combined_service.build(&serde_json::json!({}), state_model)?;

        Ok(TestTraversalModelResult {
            model,
            service: combined_service,
            input_features: vec![],
            output_features: all_output_features,
        })
    }
}

/// A mock `TraversalModelService` that provides mock outputs for all input features
/// required by a downstream service, allowing the downstream service to be tested in isolation.
pub struct MockUpstreamService {
    output_features: Vec<(String, StateVariableConfig)>,
}

impl MockUpstreamService {
    /// Creates a new mock upstream service that will provide all input features
    /// required by the given service.
    ///
    /// # Arguments
    ///
    /// * `service` - The downstream service whose inputs should be mocked
    pub fn new_from(service: Arc<dyn TraversalModelService>) -> Self {
        let output_features = service
            .input_features()
            .iter()
            .map(Self::input_feature_to_output_config)
            .collect();
        Self { output_features }
    }

    /// Converts an `InputFeature` to a `StateVariableConfig` for mocking.
    /// All mock features are initialized to zero and are non-accumulator.
    ///
    /// This is a public helper method that can be used by tests to manually
    /// construct mock state configurations.
    pub fn input_feature_to_output_config(feature: &InputFeature) -> (String, StateVariableConfig) {
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
            InputFeature::Custom { name, unit } => {
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
        }
    }
}

impl TraversalModelService for MockUpstreamService {
    fn input_features(&self) -> Vec<InputFeature> {
        // Mock service has no inputs - it generates features from nothing
        vec![]
    }

    fn output_features(&self) -> Vec<(String, StateVariableConfig)> {
        self.output_features.clone()
    }

    fn build(
        &self,
        _query: &serde_json::Value,
        _state_model: Arc<StateModel>,
    ) -> Result<Arc<dyn TraversalModel>, TraversalModelError> {
        Ok(Arc::new(MockUpstreamModel {}))
    }
}

/// A no-op `TraversalModel` used by `MockUpstreamService`.
/// This model doesn't actually modify state - it just satisfies the trait requirements.
struct MockUpstreamModel {}

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
        // No-op: mock provides initial state but doesn't update during traversal
        Ok(())
    }

    fn estimate_traversal(
        &self,
        _od: (&Vertex, &Vertex),
        _state: &mut Vec<crate::model::state::StateVariable>,
        _tree: &SearchTree,
        _state_model: &crate::model::state::StateModel,
    ) -> Result<(), TraversalModelError> {
        // No-op: mock provides initial state but doesn't update during estimation
        Ok(())
    }
}

/// A combined `TraversalModelService` that chains multiple services together.
/// Services are executed in the order provided.
struct CombinedTraversalService {
    services: Vec<Arc<dyn TraversalModelService>>,
}

impl CombinedTraversalService {
    fn new(services: Vec<Arc<dyn TraversalModelService>>) -> Self {
        Self { services }
    }
}

impl TraversalModelService for CombinedTraversalService {
    fn input_features(&self) -> Vec<InputFeature> {
        // The combined service's inputs are the union of all services' inputs
        // minus any features produced by upstream services
        let mut required_inputs = Vec::new();
        let mut provided_outputs = std::collections::HashSet::new();

        for service in &self.services {
            // Add outputs from this service to the provided set
            for (name, _) in service.output_features() {
                provided_outputs.insert(name);
            }

            // Only include inputs that aren't already provided
            for input in service.input_features() {
                let name = match &input {
                    InputFeature::Distance { name, .. } => name,
                    InputFeature::Ratio { name, .. } => name,
                    InputFeature::Speed { name, .. } => name,
                    InputFeature::Time { name, .. } => name,
                    InputFeature::Energy { name, .. } => name,
                    InputFeature::Temperature { name, .. } => name,
                    InputFeature::Custom { name, .. } => name,
                };
                if !provided_outputs.contains(name)
                    && !required_inputs.iter().any(|f| match f {
                        InputFeature::Distance { name: n, .. } => n == name,
                        InputFeature::Ratio { name: n, .. } => n == name,
                        InputFeature::Speed { name: n, .. } => n == name,
                        InputFeature::Time { name: n, .. } => n == name,
                        InputFeature::Energy { name: n, .. } => n == name,
                        InputFeature::Temperature { name: n, .. } => n == name,
                        InputFeature::Custom { name: n, .. } => n == name,
                    })
                {
                    required_inputs.push(input);
                }
            }
        }

        required_inputs
    }

    fn output_features(&self) -> Vec<(String, StateVariableConfig)> {
        // The combined service's outputs are the union of all services' outputs
        let mut all_outputs = Vec::new();
        let mut seen_names = std::collections::HashSet::new();

        for service in &self.services {
            for (name, config) in service.output_features() {
                if !seen_names.contains(&name) {
                    seen_names.insert(name.clone());
                    all_outputs.push((name, config));
                }
            }
        }

        all_outputs
    }

    fn build(
        &self,
        query: &serde_json::Value,
        state_model: Arc<StateModel>,
    ) -> Result<Arc<dyn TraversalModel>, TraversalModelError> {
        // Build models from all services and combine them
        let mut models = Vec::new();
        for service in &self.services {
            models.push(service.build(query, state_model.clone())?);
        }
        Ok(Arc::new(CombinedTraversalModel::new(models)))
    }
}
