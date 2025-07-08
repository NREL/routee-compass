use uom::ConstZero;

use crate::model::state::{CustomFeatureFormat, InputFeature};
use crate::model::traversal::TraversalModel;
use crate::model::{
    state::StateFeature,
    traversal::{default::combined::CombinedTraversalModel, TraversalModelError},
};
use std::sync::Arc;

/// a traversal model that can be used in tests which will "plug in" a mock model that
/// registers all of the input feature requirements for the provided model.
pub struct TestTraversalModel {}

impl TestTraversalModel {
    /// build (wrap) the model for testing.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        model: Arc<dyn TraversalModel>,
    ) -> Result<Arc<dyn TraversalModel>, TraversalModelError> {
        let upstream: Box<dyn TraversalModel> =
            Box::new(MockUpstreamModel::new_upstream_from(model.clone()));
        let wrapped: Arc<dyn TraversalModel> = Arc::new(CombinedTraversalModel::new(vec![
            Arc::from(upstream),
            model.clone(),
        ]));
        Ok(wrapped)
    }
}

struct MockUpstreamModel {
    input_features: Vec<InputFeature>,
    output_features: Vec<(String, StateFeature)>,
}

impl MockUpstreamModel {
    /// builds a new mock upstream TraversalModel that registers all of the input feature
    /// requirements for the provided model.
    pub fn new_upstream_from(model: Arc<dyn TraversalModel>) -> MockUpstreamModel {
        let input_features = vec![];
        let output_features = model
            .input_features()
            .iter()
            .map(|feature| match feature {
                InputFeature::Distance { name, unit: _ } => (
                    name.clone(),
                    StateFeature::Distance {
                        value: uom::si::f64::Length::ZERO,
                        accumulator: false,
                        output_unit: None,
                    },
                ),
                InputFeature::Ratio { name, unit: _ } => (
                    name.clone(),
                    StateFeature::Ratio {
                        value: uom::si::f64::Ratio::ZERO,
                        accumulator: false,
                        output_unit: None,
                    },
                ),
                InputFeature::Speed { name, unit: _ } => (
                    name.clone(),
                    StateFeature::Speed {
                        value: uom::si::f64::Velocity::ZERO,
                        accumulator: false,
                        output_unit: None,
                    },
                ),
                InputFeature::Time { name, unit: _ } => (
                    name.clone(),
                    StateFeature::Time {
                        value: uom::si::f64::Time::ZERO,
                        accumulator: false,
                        output_unit: None,
                    },
                ),
                InputFeature::Energy { name, unit: _ } => (
                    name.clone(),
                    StateFeature::Energy {
                        value: uom::si::f64::Energy::ZERO,
                        accumulator: false,
                        output_unit: None,
                    },
                ),
                InputFeature::Custom { name, unit: _ } => (
                    name.clone(),
                    StateFeature::Custom {
                        value: 0.0,
                        accumulator: false,
                        format: CustomFeatureFormat::FloatingPoint {
                            initial: ordered_float::OrderedFloat(0.0),
                        },
                    },
                ),
            })
            .collect();
        Self {
            input_features,
            output_features,
        }
    }
}

impl TraversalModel for MockUpstreamModel {
    fn name(&self) -> String {
        String::from("Mock Upstream Traversal Model")
    }
    fn input_features(&self) -> Vec<InputFeature> {
        self.input_features.clone()
    }

    fn output_features(&self) -> Vec<(String, StateFeature)> {
        self.output_features.clone()
    }

    fn traverse_edge(
        &self,
        _trajectory: (
            &crate::model::network::Vertex,
            &crate::model::network::Edge,
            &crate::model::network::Vertex,
        ),
        _state: &mut Vec<crate::model::state::StateVariable>,
        _state_model: &crate::model::state::StateModel,
    ) -> Result<(), TraversalModelError> {
        Ok(())
    }

    fn estimate_traversal(
        &self,
        _od: (
            &crate::model::network::Vertex,
            &crate::model::network::Vertex,
        ),
        _state: &mut Vec<crate::model::state::StateVariable>,
        _state_model: &crate::model::state::StateModel,
    ) -> Result<(), TraversalModelError> {
        Ok(())
    }
}
