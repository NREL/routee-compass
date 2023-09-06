use super::termination_model_error::TerminationModelError;
use crate::util::duration_extension::DurationExtension;
use serde::Deserialize;
use std::time::{Duration, Instant};

#[derive(Debug, Deserialize)]
pub enum TerminationModel {
    #[serde(rename = "query_runtime")]
    QueryRuntimeLimit { limit: Duration, frequency: u64 },
    #[serde(rename = "solution_size")]
    SolutionSizeLimit { limit: usize },
    #[serde(rename = "iterations")]
    IterationsLimit { limit: u64 },
    #[serde(rename = "combined")]
    Combined { models: Vec<TerminationModel> },
}

impl TerminationModel {
    /// predicate to test whether a query should terminate based on
    /// application-level configurations
    pub fn terminate_search(
        &self,
        start_time: &Instant,
        solution_size: usize,
        iterations: u64,
    ) -> Result<bool, TerminationModelError> {
        use TerminationModel as T;
        match self {
            T::QueryRuntimeLimit { limit, frequency } => {
                if iterations % frequency == 0 {
                    let dur = Instant::now().duration_since(*start_time);
                    Ok(dur < *limit)
                } else {
                    Ok(false)
                }
            }
            T::SolutionSizeLimit { limit } => Ok(solution_size < *limit),
            T::IterationsLimit { limit } => Ok(iterations < *limit),
            T::Combined { models } => models.iter().try_fold(false, |acc, m| {
                m.terminate_search(start_time, solution_size, iterations)
                    .map(|r| acc && r)
            }),
        }
    }

    /// assuming the termination model has
    pub fn explain_termination(
        &self,
        start_time: &Instant,
        solution_size: usize,
        iterations: u64,
    ) -> Option<String> {
        use TerminationModel as T;
        let caused_termination = self
            .terminate_search(start_time, solution_size, iterations)
            .unwrap_or(false);
        match self {
            T::Combined { models } => models
                .iter()
                .map(|m| m.explain_termination(start_time, solution_size, iterations))
                .collect::<Option<Vec<_>>>()
                .map(|result| result.join(", ")),
            T::QueryRuntimeLimit { limit, .. } => {
                if caused_termination {
                    Some(format!("exceeded runtime limit of {}", limit.hhmmss()))
                } else {
                    None
                }
            }
            T::SolutionSizeLimit { limit } => {
                if caused_termination {
                    Some(format!("exceeded solution size limit of {}", limit))
                } else {
                    None
                }
            }
            T::IterationsLimit { limit } => {
                if caused_termination {
                    Some(format!("exceeded iteration limit of {}", limit))
                } else {
                    None
                }
            }
        }
    }
}
