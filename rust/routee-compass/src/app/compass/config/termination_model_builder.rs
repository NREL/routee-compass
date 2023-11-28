use super::compass_configuration_error::CompassConfigurationError;
use crate::app::compass::config::compass_configuration_field::CompassConfigurationField;
use crate::app::compass::config::config_json_extension::ConfigJsonExtensions;
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
        scope: Option<String>,
    ) -> Result<TerminationModel, CompassConfigurationError> {
        use routee_compass_core::model::termination::termination_model::TerminationModel as T;
        let local_scope = scope.unwrap_or(CompassConfigurationField::Termination.to_string());
        let term_type = config.get_config_string(String::from("type"), local_scope.clone())?;

        let result = match term_type.to_lowercase().as_str() {
            "query_runtime" => {
                let dur_val = config.get("limit").ok_or(
                    CompassConfigurationError::ExpectedFieldForComponent(
                        local_scope.clone(),
                        String::from("limit"),
                    ),
                )?;
                let dur = dur_val.as_duration()?;
                let freq =
                    config.get_config_i64(String::from("frequency"), local_scope.clone())? as u64;
                Ok(T::QueryRuntimeLimit {
                    limit: dur,
                    frequency: freq,
                })
            }
            "iterations" => {
                let iterations =
                    config.get_config_i64(String::from("limit"), local_scope.clone())? as u64;
                Ok(T::IterationsLimit { limit: iterations })
            }
            "solution_size" => {
                let solution_size =
                    config.get_config_i64(String::from("limit"), local_scope.clone())? as usize;
                Ok(T::SolutionSizeLimit {
                    limit: solution_size,
                })
            }
            "combined" => {
                let models_val =
                    config.get_config_array(String::from("models"), local_scope.clone())?;

                let models = models_val
                    .iter()
                    .enumerate()
                    .map(|(idx, c)| {
                        let next_scope = format!("{}.combined[{}]", local_scope.clone(), idx);
                        TerminationModelBuilder::build(c, Some(next_scope))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(T::Combined { models })
            }
            _ => Err(CompassConfigurationError::UnknownModelNameForComponent(
                term_type,
                local_scope,
                String::from("query_runtime, iterations, solution_size, combined"),
            )),
        }?;

        log::info!("app termination model: {:?}", result);
        Ok(result)
    }
}
