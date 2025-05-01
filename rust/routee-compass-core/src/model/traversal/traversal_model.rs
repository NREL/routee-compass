use super::traversal_model_error::TraversalModelError;
use crate::model::network::{Edge, Vertex};
use crate::model::state::StateModel;
use crate::model::state::StateVariable;
use crate::model::state::{InputFeature, OutputFeature};

/// Dictates how state transitions occur while traversing a graph in a search algorithm.
///
/// see the [`super::default`] module for implementations bundled with RouteE Compass:
///   - [DistanceModel]: uses Edge distances to find the route with the shortest distance
///   - [SpeedLookupModel]: retrieves link speeds via lookup from a file
///
/// [DistanceModel]: super::default::distance::DistanceModel
/// [SpeedLookupModel]: super::default::speed_lookup_model::SpeedLookupModel
pub trait TraversalModel: Send + Sync {
    /// list the state variables required as inputs to this traversal model. for
    /// example, if this traversal model uses a distance metric to compute time, then
    /// it should list the expected distance state variable here.
    fn input_features(&self) -> Vec<(String, InputFeature)>;

    /// lists the state variables produced by this traversal model. for example,
    /// if this traversal model produces leg distances, it should specify that here.
    fn output_features(&self) -> Vec<(String, OutputFeature)>;

    /// Updates the traversal state by traversing an edge.
    ///
    /// # Arguments
    ///
    /// * `trajectory` - source vertex, edge, and destination vertex
    /// * `state` - state of the search at the beginning of this edge
    /// * `state_model` - provides access to the state vector
    ///
    /// # Returns
    ///
    /// Either a traversal result or an error.
    fn traverse_edge(
        &self,
        trajectory: (&Vertex, &Edge, &Vertex),
        state: &mut Vec<StateVariable>,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError>;

    /// Estimates the traversal state by traversing between two vertices without
    /// performing any graph traversals.
    ///
    /// # Arguments
    ///
    /// * `od` - source vertex and destination vertex
    /// * `state` - state of the search at the source vertex
    /// * `state_model` - provides access to the state vector
    ///
    /// # Returns
    ///
    /// Either a traversal result or an error.
    fn estimate_traversal(
        &self,
        od: (&Vertex, &Vertex),
        state: &mut Vec<StateVariable>,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError>;
}

#[cfg(test)]
mod test {
    use super::TraversalModel;
    use crate::model::{
        state::{CustomFeatureFormat, InputFeature, OutputFeature},
        traversal::{
            default::combined::{CombinedTraversalBuilder, CombinedTraversalModel},
            TraversalModelBuilder, TraversalModelError,
        },
        unit::{DistanceUnit, EnergyUnit, GradeUnit, SpeedUnit, TimeUnit},
    };
    use itertools::Itertools;
    use serde_json::Value;
    use std::{collections::HashMap, rc::Rc, sync::Arc};

    struct MockUpstreamModel {
        output_features: Vec<String, OutputFeature>,
    }

    impl MockUpstreamModel {
        pub fn new_upstream_from(m: impl TraversalModel) -> MockUpstreamModel {
            let output_features = m
                .input_features()
                .iter()
                .map(|(_, f)| match f {
                    InputFeature::Distance(distance_unit) => {
                        let du = distance_unit.unwrap_or_else(|| DistanceUnit::Miles);
                        OutputFeature::Distance {
                            distance_unit: du,
                            initial: Distance::ZERO,
                        }
                    }
                    InputFeature::Speed(speed_unit) => {
                        let unit = speed_unit.unwrap_or_else(|| SpeedUnit::MPH);
                        OutputFeature::Speed {
                            speed_unit: unit,
                            initial: Speed::ZERO,
                        }
                    }
                    InputFeature::Time(time_unit) => {
                        let unit = time_unit.unwrap_or_else(|| TimeUnit::Hours);
                        OutputFeature::Time {
                            time_unit: unit,
                            initial: Time::ZERO,
                        }
                    }
                    InputFeature::Energy(energy_unit) => {
                        let unit = energy_unit.unwrap_or_else(|| EnergyUnit::KilowattHours);
                        OutputFeature::Energy {
                            energy_unit: unit,
                            initial: Energy::ZERO,
                        }
                    }
                    InputFeature::Grade(grade_unit) => {
                        let unit = grade_unit.unwrap_or_else(|| GradeUnit::Percent);
                        OutputFeature::Grade {
                            grade_unit: unit,
                            initial: Grade::ZERO,
                        }
                    }
                    InputFeature::Custom { r#type, unit } => {
                        let format = CustomFeatureFormat::Boolean { initial: true };
                        OutputFeature::Custom {
                            r#type: r#type.clone(),
                            unit: unit.clone(),
                            format,
                        }
                    }
                })
                .collect_vec();
            Self { output_features }
        }
    }

    impl TraversalModel for MockUpstreamModel {
        fn input_features(&self) -> Vec<(String, crate::model::state::InputFeature)> {
            vec![]
        }

        fn output_features(&self) -> Vec<(String, OutputFeature)> {
            self.outputs.to_vec()
        }

        fn traverse_edge(
            &self,
            trajectory: (
                &crate::model::network::Vertex,
                &crate::model::network::Edge,
                &crate::model::network::Vertex,
            ),
            state: &mut Vec<crate::model::state::StateVariable>,
            state_model: &crate::model::state::StateModel,
        ) -> Result<(), TraversalModelError> {
            Ok(())
        }

        fn estimate_traversal(
            &self,
            od: (
                &crate::model::network::Vertex,
                &crate::model::network::Vertex,
            ),
            state: &mut Vec<crate::model::state::StateVariable>,
            state_model: &crate::model::state::StateModel,
        ) -> Result<(), TraversalModelError> {
            Ok(())
        }
    }

    pub struct TestTraversalModel {}

    impl TestTraversalModel {
        pub fn wrap_model(
            model: impl TraversalModel,
            config: Option<&Value>,
            query: Option<&Value>,
        ) -> Result<Arc<dyn TraversalModel>, TraversalModelError> {
            // let c = config.unwrap_or_default();
            // let q = query.unwrap_or_default();
            // let b = CombinedTraversalBuilder::new(HashMap::from([String::from("dummy_name"), Rc::new(builder)]));
            // let service = b.build(c)?;
            // let model = service.build(q)?;
            let wrapped: dyn TraversalModel = CombinedTraversalModel::new(vec![Arc::new(
                MockUpstreamModel::new_upstream_from(model),
            )]);
            Ok(Arc::new(wrapped))
        }
    }
}
