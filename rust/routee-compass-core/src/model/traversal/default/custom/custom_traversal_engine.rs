use super::{sparse_read_ops, CustomTraversalConfig};
use crate::model::network::EdgeId;
use crate::model::state::{CustomFeatureFormat, StateFeature, StateModel, StateVariable};
use crate::util::fs::read_decoders;
use crate::{model::traversal::TraversalModelError, util::fs::read_utils};
use kdam::BarBuilder;
use std::collections::HashMap;

/// provides lookup capabilities for the custom feature based on sparse or dense data layout
pub enum CustomTraversalEngine {
    /// for each EdgeId in the graph, there exists a custom feature value
    Dense {
        config: CustomTraversalConfig,
        values: Box<[StateVariable]>,
    },
    /// for a subset of EdgeIds in the graph, there exists a custom feature value
    Sparse {
        config: CustomTraversalConfig,
        values: HashMap<EdgeId, StateVariable>,
    },
}

impl CustomTraversalEngine {
    pub fn config(&self) -> &CustomTraversalConfig {
        match self {
            CustomTraversalEngine::Dense { config, .. } => config,
            CustomTraversalEngine::Sparse { config, .. } => config,
        }
    }

    pub fn output_feature(&self) -> StateFeature {
        let config = self.config();
        StateFeature::Custom {
            value: 0.0,
            accumulator: config.accumulator,
            format: config.feature,
        }
    }

    pub fn insert_value(
        &self,
        edge_id: &EdgeId,
        state: &mut [StateVariable],
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        let (value, config) = match self {
            CustomTraversalEngine::Dense { values, config } => (values.get(edge_id.0), config),
            CustomTraversalEngine::Sparse { values, config } => (values.get(edge_id), config),
        };
        let found_value = value.ok_or_else(|| {
            TraversalModelError::TraversalModelFailure(format!(
                "edge id {} not found in custom feature model with configuration: {}",
                edge_id, config
            ))
        })?;
        match config.feature {
            CustomFeatureFormat::FloatingPoint { .. } => {
                let mut value = config.feature.decode_f64(found_value)?;
                if config.accumulator {
                    let prev = state_model.get_custom_f64(state, &config.name)?;
                    value += prev;
                }
                state_model.set_custom_f64(state, &config.name, &value)?;
            }
            CustomFeatureFormat::SignedInteger { .. } => {
                let mut value = config.feature.decode_i64(found_value)?;
                if config.accumulator {
                    let prev = state_model.get_custom_i64(state, &config.name)?;
                    value += prev;
                }
                state_model.set_custom_i64(state, &config.name, &value)?;
            }
            CustomFeatureFormat::UnsignedInteger { .. } => {
                let mut value = config.feature.decode_u64(found_value)?;
                if config.accumulator {
                    let prev = state_model.get_custom_u64(state, &config.name)?;
                    value += prev;
                }
                state_model.set_custom_u64(state, &config.name, &value)?;
            }
            CustomFeatureFormat::Boolean { .. } => {
                let mut value = config.feature.decode_bool(found_value)?;
                if config.accumulator {
                    let prev = state_model.get_custom_bool(state, &config.name)?;
                    value = value && prev;
                }
                state_model.set_custom_bool(state, &config.name, &value)?;
            }
        }
        Ok(())
    }
}

impl TryFrom<&CustomTraversalConfig> for CustomTraversalEngine {
    type Error = TraversalModelError;

    fn try_from(config: &CustomTraversalConfig) -> Result<Self, Self::Error> {
        let bar_builder = BarBuilder::default().desc(config.input_file.to_string());

        match config.file_format {
            super::CustomInputFormat::Dense => {
                use CustomFeatureFormat as C;
                let decoder = match config.feature {
                    C::FloatingPoint { .. } => read_decoders::state_variable,
                    C::SignedInteger { .. } => read_decoders::i64_to_state_variable,
                    C::UnsignedInteger { .. } => read_decoders::u64_to_state_variable,
                    C::Boolean { .. } => read_decoders::bool_to_state_variable,
                };
                let values: Box<[StateVariable]> =
                    read_utils::read_raw_file(&config.input_file, decoder, Some(bar_builder), None)
                        .map_err(|e| {
                            TraversalModelError::BuildError(format!(
                                "failure reading custom input file {}: {}",
                                config.input_file, e
                            ))
                        })?;
                Ok(Self::Dense {
                    values,
                    config: config.clone(),
                })
            }
            super::CustomInputFormat::Sparse => {
                let values = match config.feature {
                    CustomFeatureFormat::Boolean { .. } => {
                        sparse_read_ops::read_bools(&config.input_file, bar_builder)
                    }
                    _ => sparse_read_ops::read_state_variables(&config.input_file, bar_builder),
                }?;
                Ok(Self::Sparse {
                    values,
                    config: config.clone(),
                })
            }
        }
    }
}
