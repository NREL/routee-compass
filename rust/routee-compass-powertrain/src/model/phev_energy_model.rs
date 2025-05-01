use super::prediction::PredictionModelRecord;
use routee_compass_core::model::{
    network::{Edge, Vertex},
    state::{InputFeature, OutputFeature, StateModel, StateVariable},
    traversal::{TraversalModel, TraversalModelError},
};
use std::sync::Arc;

pub struct PhevEnergyModel {
    pub charge_sustain_model: Arc<PredictionModelRecord>,
    pub charge_depleting_model: Arc<PredictionModelRecord>,
}

impl PhevEnergyModel {
    const EDGE_ENERGY_LIQUID: &'static str = "edge_energy_liquid";
    const TRIP_ENERGY_LIQUID: &'static str = "trip_energy_liquid";
    const EDGE_ENERGY_ELECTRIC: &'static str = "edge_energy_electric";
    const TRIP_ENERGY_ELECTRIC: &'static str = "trip_energy_electric";
    const EDGE_DISTANCE: &'static str = "edge_distance";
    const EDGE_SPEED: &'static str = "edge_speed";
    const EDGE_GRADE: &'static str = "edge_grade";
}

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
