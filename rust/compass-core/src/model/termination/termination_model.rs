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
                    let result = dur > *limit;
                    Ok(result)
                } else {
                    Ok(false)
                }
            }
            T::SolutionSizeLimit { limit } => Ok(solution_size > *limit),
            T::IterationsLimit { limit } => Ok(iterations > *limit),
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

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::TerminationModel;

    #[test]
    fn test_within_runtime_limit() {
        let within_limit = Duration::from_secs(1);
        let start_time = Instant::now() - within_limit;
        let limit = Duration::from_secs(2);
        let frequency = 10;

        let m = TerminationModel::QueryRuntimeLimit { limit, frequency };
        for iteration in 0..(frequency + 1) {
            let result = m.terminate_search(&start_time, 0, iteration).unwrap();
            // in all iterations, the result should be false, though for iterations 1-9, that will be due to the sample frequency
            assert_eq!(result, false);
        }
    }

    #[test]
    fn test_exceeds_runtime_limit() {
        let exceeds_limit = Duration::from_secs(3);
        let start_time = Instant::now() - exceeds_limit;
        let limit = Duration::from_secs(2);
        let frequency = 10;

        let m = TerminationModel::QueryRuntimeLimit { limit, frequency };
        for iteration in 0..(frequency + 1) {
            let result = m.terminate_search(&start_time, 0, iteration).unwrap();
            if iteration == 0 {
                // edge case. when iteration == 0, we will run the test, and it should fail, since 10 % 0 == 0 is true.
                // but let's continue testing iterations 1-10 to explore the expected range of behaviors.
                assert_eq!(result, true);
            } else if iteration != frequency {
                // from iterations 1 to 9, terminate is false because of the frequency argument of 10
                // bypasses the runtime test
                assert_eq!(result, false);
            } else {
                // on iteration 10, terminate is true because "exceeds_limit_time" is greater than the limit duration
                assert_eq!(result, true);
            }
        }
    }
}
