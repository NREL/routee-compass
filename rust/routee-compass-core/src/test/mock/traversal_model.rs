use crate::model::traversal::TraversalModel;
use crate::model::{
    state::{CustomFeatureFormat, InputFeature, OutputFeature},
    traversal::{default::combined::CombinedTraversalModel, TraversalModelError},
    unit::*,
};
use itertools::Itertools;
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
    input_features: Vec<(String, InputFeature)>,
    output_features: Vec<(String, OutputFeature)>,
}

impl MockUpstreamModel {
    /// builds a new mock upstream TraversalModel that registers all of the input feature
    /// requirements for the provided model.
    pub fn new_upstream_from(model: Arc<dyn TraversalModel>) -> MockUpstreamModel {
        // let input_features = model.input_features();
        let input_features = vec![];
        let output_features = model
            .input_features()
            .iter()
            .map(|(n, f)| match f {
                InputFeature::Distance(distance_unit) => {
                    let du = distance_unit.unwrap_or_else(|| DistanceUnit::Miles);
                    (
                        n.clone(),
                        OutputFeature::Distance {
                            distance_unit: du,
                            initial: Distance::ZERO,
                            accumulator: true,
                        },
                    )
                }
                InputFeature::Speed(speed_unit) => {
                    let unit = speed_unit.unwrap_or_else(|| SpeedUnit::MPH);
                    (
                        n.clone(),
                        OutputFeature::Speed {
                            speed_unit: unit,
                            initial: Speed::ZERO,
                            accumulator: true,
                        },
                    )
                }
                InputFeature::Time(time_unit) => {
                    let unit = time_unit.unwrap_or_else(|| TimeUnit::Hours);
                    (
                        n.clone(),
                        OutputFeature::Time {
                            time_unit: unit,
                            initial: Time::ZERO,
                            accumulator: true,
                        },
                    )
                }
                InputFeature::Energy(energy_unit) => {
                    let unit = energy_unit.unwrap_or_else(|| EnergyUnit::KilowattHours);
                    (
                        n.clone(),
                        OutputFeature::Energy {
                            energy_unit: unit,
                            initial: Energy::ZERO,
                            accumulator: true,
                        },
                    )
                }
                InputFeature::Grade(grade_unit) => {
                    let unit = grade_unit.unwrap_or_else(|| GradeUnit::Percent);
                    (
                        n.clone(),
                        OutputFeature::Grade {
                            grade_unit: unit,
                            initial: Grade::ZERO,
                            accumulator: true,
                        },
                    )
                }
                InputFeature::Custom { r#type, unit } => {
                    let format = CustomFeatureFormat::FloatingPoint {
                        initial: 0.0.into(),
                    };
                    (
                        n.clone(),
                        OutputFeature::Custom {
                            r#type: r#type.clone(),
                            unit: unit.clone(),
                            format,
                            accumulator: true,
                        },
                    )
                }
            })
            .collect_vec();
        Self {
            input_features,
            output_features,
        }
    }
}

impl TraversalModel for MockUpstreamModel {
    fn input_features(&self) -> Vec<(String, crate::model::state::InputFeature)> {
        self.input_features.to_vec()
    }

    fn output_features(&self) -> Vec<(String, OutputFeature)> {
        self.output_features.to_vec()
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
