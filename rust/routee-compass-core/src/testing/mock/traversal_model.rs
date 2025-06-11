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
    input_features: Vec<String>,
    output_features: Vec<(String, StateFeature)>,
}

impl MockUpstreamModel {
    /// builds a new mock upstream TraversalModel that registers all of the input feature
    /// requirements for the provided model.
    pub fn new_upstream_from(model: Arc<dyn TraversalModel>) -> MockUpstreamModel {
        let input_features = vec![];
        let output_features = model.output_features();
        Self {
            input_features,
            output_features,
        }
    }
}

impl TraversalModel for MockUpstreamModel {
    fn input_features(&self) -> Vec<String> {
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
