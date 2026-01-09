use uom::si::f64::ThermodynamicTemperature;

use crate::{
    algorithm::search::SearchTree,
    model::{
        network::{Edge, Vertex},
        state::{StateModel, StateVariable},
        traversal::{default::fieldname, TraversalModel, TraversalModelError},
    },
};

#[derive(Clone, Debug)]
pub struct TemperatureTraversalModel {
    pub ambient_temperature: ThermodynamicTemperature,
}

impl TraversalModel for TemperatureTraversalModel {
    fn name(&self) -> String {
        String::from("Temperature Traversal Model")
    }

    fn traverse_edge(
        &self,
        _trajectory: (&Vertex, &Edge, &Vertex),
        state: &mut Vec<StateVariable>,
        _tree: &SearchTree,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        state_model.set_temperature(
            state,
            fieldname::AMBIENT_TEMPERATURE,
            &self.ambient_temperature,
        )?;
        Ok(())
    }

    fn estimate_traversal(
        &self,
        _od: (&Vertex, &Vertex),
        _state: &mut Vec<StateVariable>,
        _tree: &SearchTree,
        _state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        Ok(())
    }
}
