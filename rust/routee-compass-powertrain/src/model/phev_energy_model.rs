use routee_compass_core::model::{
    network::{Edge, Vertex},
    state::{InputFeature, OutputFeature, StateModel, StateVariable},
    traversal::{TraversalModel, TraversalModelError},
};

pub struct PhevEnergyModel {}

impl TraversalModel for PhevEnergyModel {
    fn input_features(&self) -> Vec<(String, InputFeature)> {
        todo!()
    }

    fn output_features(&self) -> Vec<(String, OutputFeature)> {
        todo!()
    }

    fn traverse_edge(
        &self,
        trajectory: (&Vertex, &Edge, &Vertex),
        state: &mut Vec<StateVariable>,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        todo!()
    }

    fn estimate_traversal(
        &self,
        od: (&Vertex, &Vertex),
        state: &mut Vec<StateVariable>,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        todo!()
    }
}
