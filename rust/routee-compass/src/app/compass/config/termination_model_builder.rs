use super::compass_configuration_error::CompassConfigurationError;
use crate::app::compass::config::compass_configuration_field::CompassConfigurationField;
use log;
use routee_compass_core::model::termination::termination_model::TerminationModel;
use routee_compass_core::util::conversion::duration_extension::DurationExtension;

pub struct TerminationModelBuilder {}

impl TerminationModelBuilder {
    /// implementing deserialization for TerminationModels like this has the
    /// smell of rebuilding the codec wheel, but doing it like this provides
    /// consistency with our user interfaces.
    pub fn build(
        config: &serde_json::Value,
    ) -> Result<TerminationModel, CompassConfigurationError> {
        use routee_compass_core::model::termination::termination_model::TerminationModel as T;
        let term_type_obj =
            config
                .get("type")
                .ok_or(CompassConfigurationError::ExpectedFieldForComponent(
                    CompassConfigurationField::Termination.to_string(),
                    String::from("type"),
                ))?;
        let term_type: String = term_type_obj
            .as_str()
            .ok_or(CompassConfigurationError::ExpectedFieldWithType(
                String::from("type"),
                String::from("String"),
            ))?
            .into();

        let result =
            match term_type.to_lowercase().as_str() {
                "query_runtime" => {
                    let dur_val = config.get("limit").ok_or(
                        CompassConfigurationError::ExpectedFieldForComponent(
                            CompassConfigurationField::Termination.to_string(),
                            String::from("limit"),
                        ),
                    )?;
                    let dur = dur_val.as_duration()?;
                    let freq_val = config.get("frequency").ok_or(
                        CompassConfigurationError::ExpectedFieldForComponent(
                            CompassConfigurationField::Termination.to_string(),
                            String::from("frequency"),
                        ),
                    )?;
                    let freq = freq_val.as_u64().ok_or(
                        CompassConfigurationError::ExpectedFieldWithType(
                            String::from("frequency"),
                            String::from("integer"),
                        ),
                    )?;
                    Ok(T::QueryRuntimeLimit {
                        limit: dur,
                        frequency: freq,
                    })
                }
                "iterations" => {
                    let iters_val = config.get("limit").ok_or(
                        CompassConfigurationError::ExpectedFieldForComponent(
                            CompassConfigurationField::Termination.to_string(),
                            String::from("limit"),
                        ),
                    )?;
                    let iterations = iters_val.as_u64().ok_or(
                        CompassConfigurationError::ExpectedFieldWithType(
                            String::from("limit"),
                            String::from("integer"),
                        ),
                    )?;
                    Ok(T::IterationsLimit { limit: iterations })
                }
                "solution_size" => {
                    let size_val = config.get("limit").ok_or(
                        CompassConfigurationError::ExpectedFieldForComponent(
                            CompassConfigurationField::Termination.to_string(),
                            String::from("limit"),
                        ),
                    )?;
                    let solution_size = size_val.as_u64().ok_or(
                        CompassConfigurationError::ExpectedFieldWithType(
                            String::from("limit"),
                            String::from("integer"),
                        ),
                    )?;
                    Ok(T::SolutionSizeLimit {
                        limit: solution_size as usize,
                    })
                }
                "combined" => {
                    let models_val = config
                        .get("models")
                        .ok_or(CompassConfigurationError::ExpectedFieldForComponent(
                            CompassConfigurationField::Termination.to_string(),
                            String::from("models"),
                        ))?
                        .as_array()
                        .ok_or(CompassConfigurationError::ExpectedFieldWithType(
                            String::from("models"),
                            String::from("JSON array"),
                        ))?;
                    let models = models_val
                        .into_iter()
                        .map(|c| TerminationModelBuilder::build(c))
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok(T::Combined { models })
                }
                _ => Err(CompassConfigurationError::UnknownModelNameForComponent(
                    term_type,
                    CompassConfigurationField::Termination.to_string(),
                )),
            }?;

        log::info!("app termination model: {:?}", result);
        return Ok(result);
    }
}
