use crate::model::network::{Edge, Vertex};
use crate::model::state::StateModel;
use crate::model::state::StateVariable;
use crate::model::state::{InputFeature, OutputFeature};
use crate::model::traversal::traversal_model::TraversalModel;
use crate::model::traversal::traversal_model_error::TraversalModelError;
use std::sync::Arc;

use super::CustomTraversalEngine;

pub struct CustomTraversalModel {
    engine: Arc<CustomTraversalEngine>,
}

impl CustomTraversalModel {
    pub fn new(engine: Arc<CustomTraversalEngine>) -> CustomTraversalModel {
        Self { engine }
    }
}

impl TraversalModel for CustomTraversalModel {
    fn input_features(&self) -> Vec<(String, InputFeature)> {
        vec![]
    }

    fn output_features(&self) -> Vec<(String, OutputFeature)> {
        let feature = self.engine.output_feature();
        let name = feature.get_feature_unit_name();
        vec![(name, feature)]
    }

    /// records the speed that will be driven over this edge into the state vector.
    fn traverse_edge(
        &self,
        trajectory: (&Vertex, &Edge, &Vertex),
        state: &mut Vec<StateVariable>,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        let (_, edge, _) = trajectory;
        self.engine.insert_value(&edge.edge_id, state, state_model)
    }

    /// (over-)estimates speed over remainder of the trip as the maximum-possible speed value.
    fn estimate_traversal(
        &self,
        _od: (&Vertex, &Vertex),
        _state: &mut Vec<StateVariable>,
        _state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        Ok(())
    }
}
