use super::termination_model_error::TerminationModelError;
use crate::util::duration_extension::DurationExtension;
use serde::Deserialize;
use std::time::{Duration, Instant};

/// the termination model for the application should be evaluated at the top of each iteration
/// of a search. if it returns true, an error response should be created for the user using the
/// explain method.
#[derive(Debug, Deserialize)]
pub enum TerminationModel {
    /// terminates a query if the runtime exceeds some limit.
    /// only checks at some provided iteration frequency, since the computation is expensive.
    #[serde(rename = "query_runtime")]
    QueryRuntimeLimit { limit: Duration, frequency: u64 },
    /// terminates if the size of the solution exceeds (greater than) some limit
    #[serde(rename = "solution_size")]
    SolutionSizeLimit { limit: usize },
    /// terminates if the number of iterations exceeds (greater than) some limit
    /// iterations begin at 0, so we add 1 to the iteration to make this comparison
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
        iteration: u64,
    ) -> Result<bool, TerminationModelError> {
        use TerminationModel as T;
        match self {
            T::QueryRuntimeLimit { limit, frequency } => {
                if iteration % frequency == 0 {
                    let dur = Instant::now().duration_since(*start_time);
                    Ok(dur > *limit)
                } else {
                    Ok(false)
                }
            }
            T::SolutionSizeLimit { limit } => Ok(solution_size > *limit),
            T::IterationsLimit { limit } => Ok(iteration + 1 > *limit),
            T::Combined { models } => models.iter().try_fold(false, |acc, m| {
                m.terminate_search(start_time, solution_size, iteration)
                    .map(|r| acc || r)
            }),
        }
    }

    /// this method will a string explaining why a model terminated. if the
    /// conditions do not merit termination, then the result will be None.
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
            T::Combined { models } => {
                let combined_explanations: String = models
                    .iter()
                    .filter_map(|m| m.explain_termination(start_time, solution_size, iterations))
                    .collect::<Vec<_>>()
                    .join(", ");
                if combined_explanations.is_empty() {
                    None
                } else {
                    Some(combined_explanations)
                }
            }
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

    use super::TerminationModel as T;

    #[test]
    fn test_within_runtime_limit() {
        let within_limit = Duration::from_secs(1);
        let start_time = Instant::now() - within_limit;
        let limit = Duration::from_secs(2);
        let frequency = 10;

        let m = T::QueryRuntimeLimit { limit, frequency };
        for iteration in 0..(frequency + 1) {
            let result = m.terminate_search(&start_time, 0, iteration).unwrap();
            // in all iterations, the result should be false, though for iterations 1-9, that will be due to the sample frequency
            assert!(!result);
        }
    }

    #[test]
    fn test_exceeds_runtime_limit() {
        let exceeds_limit = Duration::from_secs(3);
        let start_time = Instant::now() - exceeds_limit;
        let limit = Duration::from_secs(2);
        let frequency = 10;

        let m = T::QueryRuntimeLimit { limit, frequency };
        for iteration in 0..(frequency + 1) {
            let result = m.terminate_search(&start_time, 0, iteration).unwrap();
            if iteration == 0 {
                // edge case. when iteration == 0, we will run the test, and it should fail, since 10 % 0 == 0 is true.
                // but let's continue testing iterations 1-10 to explore the expected range of behaviors.
                assert!(result);
            } else if iteration != frequency {
                // from iterations 1 to 9, terminate is false because of the frequency argument of 10
                // bypasses the runtime test
                assert!(!result);
            } else {
                // on iteration 10, terminate is true because "exceeds_limit_time" is greater than the limit duration
                assert!(result);
            }
        }
    }

    #[test]
    fn test_iterations_limit() {
        let m = T::IterationsLimit { limit: 5 };
        let i = Instant::now();
        let t_good = m.terminate_search(&i, 4, 4).unwrap();
        let t_bad1 = m.terminate_search(&i, 5, 5).unwrap();
        let t_bad2 = m.terminate_search(&i, 6, 6).unwrap();
        assert!(!t_good);
        assert!(t_bad1);
        assert!(t_bad2);
    }

    #[test]
    fn test_size_limit() {
        let m = T::SolutionSizeLimit { limit: 5 };
        let i = Instant::now();
        let t_good = m.terminate_search(&i, 4, 4).unwrap();
        let t_bad1 = m.terminate_search(&i, 5, 5).unwrap();
        let t_bad2 = m.terminate_search(&i, 6, 6).unwrap();
        assert!(!t_good);
        assert!(!t_bad1);
        assert!(t_bad2);
    }

    #[test]
    fn test_combined_3() {
        let exceeds_limit = Duration::from_secs(3);
        let start_time = Instant::now() - exceeds_limit;
        let runtime_limit = Duration::from_secs(2);
        let frequency = 1;
        let iteration_limit = 5;
        let solution_limit = 3;

        let m1 = T::QueryRuntimeLimit {
            limit: runtime_limit,
            frequency,
        };
        let m2 = T::IterationsLimit {
            limit: iteration_limit,
        };
        let m3 = T::SolutionSizeLimit {
            limit: solution_limit,
        };
        let cm = T::Combined {
            models: vec![m1, m2, m3],
        };
        let terminate = cm
            .terminate_search(&start_time, solution_limit + 1, iteration_limit + 1)
            .unwrap();
        assert!(terminate);
        let msg = cm.explain_termination(&start_time, solution_limit + 1, iteration_limit + 1);
        let expected = Some(
            [
                "exceeded runtime limit of 0:00:02.000",
                "exceeded iteration limit of 5",
                "exceeded solution size limit of 3",
            ]
            .join(", "),
        );
        assert_eq!(msg, expected);
    }

    #[test]
    fn test_combined_2_of_3() {
        let exceeds_limit = Duration::from_secs(3);
        let start_time = Instant::now() - exceeds_limit;
        let runtime_limit = Duration::from_secs(2);
        let frequency = 1;
        let iteration_limit = 5;
        let solution_limit = 3;

        let m1 = T::QueryRuntimeLimit {
            limit: runtime_limit,
            frequency,
        };
        let m2 = T::IterationsLimit {
            limit: iteration_limit,
        };
        let m3 = T::SolutionSizeLimit {
            limit: solution_limit,
        };
        let cm = T::Combined {
            models: vec![m1, m2, m3],
        };
        let terminate = cm
            .terminate_search(&start_time, solution_limit - 1, iteration_limit + 1)
            .unwrap();
        assert!(terminate);
        let msg = cm.explain_termination(&start_time, solution_limit - 1, iteration_limit + 1);
        let expected = Some(
            [
                "exceeded runtime limit of 0:00:02.000",
                "exceeded iteration limit of 5",
            ]
            .join(", "),
        );
        assert_eq!(msg, expected);
    }
}
