use super::termination_model_error::TerminationModelError;
use crate::{algorithm::search::SearchTree, util::duration_extension::DurationExtension};
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
    /// Tests if the search should terminate. If it should terminate, return a message
    /// explaining the reason to terminate. If it should not terminate, return None.
    pub fn continue_or_explain(
        &self,
        start_time: &Instant,
        solution: &SearchTree,
        iterations: u64,
    ) -> Option<String> {
        let should_terminate = self.should_terminate(start_time, solution, iterations);
        if should_terminate {
            let explanation = self.explain(start_time, solution, iterations);
            Some(explanation)
        } else {
            None
        }
    }

    /// Tests if the search should terminate. If it should terminate, generate a useful
    /// termination message and return that in the error channel. If it should not terminate,
    /// returns Ok(()).
    pub fn continue_or_error(
        &self,
        start_time: &Instant,
        solution: &SearchTree,
        iterations: u64,
    ) -> Result<(), TerminationModelError> {
        let should_terminate = self.should_terminate(start_time, solution, iterations);
        if should_terminate {
            let explanation = self.explain(start_time, solution, iterations);
            return Err(TerminationModelError::QueryTerminated(explanation));
        }
        Ok(())
    }

    /// predicate to test whether a query should terminate based on application-level configurations.
    /// evaluates before traversing a link.
    pub fn should_terminate(
        &self,
        start_time: &Instant,
        solution: &SearchTree,
        iteration: u64,
    ) -> bool {
        use TerminationModel as T;
        match self {
            T::QueryRuntimeLimit { limit, frequency } => {
                if iteration % frequency == 0 {
                    let dur = Instant::now().duration_since(*start_time);
                    dur > *limit
                } else {
                    false
                }
            }
            T::SolutionSizeLimit { limit } => {
                // if you add one more branch to the tree it would violate this termination criteria
                solution.len() >= *limit
            }
            T::IterationsLimit { limit } => {
                // if you perform one more iteration it would violate this termination criteria
                iteration >= *limit
            }
            T::Combined { models } => models.iter().fold(false, |acc, m| {
                let inner = m.should_terminate(start_time, solution, iteration);
                acc || inner
            }),
        }
    }

    /// this method will a string explaining why a model terminated. if the
    /// conditions do not merit termination, then the result will be None.
    pub fn explain(&self, start_time: &Instant, solution: &SearchTree, iterations: u64) -> String {
        use TerminationModel as T;
        // must test if this particular [`TerminationModel`] variant instance was the cause of
        // termination, in the case of [`TerminationModel::Combined`].
        let caused_termination = self.should_terminate(start_time, solution, iterations);
        match self {
            T::Combined { models } => {
                let combined_explanations: String = models
                    .iter()
                    .map(|m| m.explain(start_time, solution, iterations))
                    .filter(|m| !m.is_empty())
                    .collect::<Vec<_>>()
                    .join(", ");
                if combined_explanations.is_empty() {
                    "".to_string()
                } else {
                    combined_explanations
                }
            }
            T::QueryRuntimeLimit { limit, .. } => {
                if caused_termination {
                    format!("exceeded runtime limit of {}", limit.hhmmss())
                } else {
                    "".to_string()
                }
            }
            T::SolutionSizeLimit { limit } => {
                if caused_termination {
                    format!("exceeded solution size limit of {limit}")
                } else {
                    "".to_string()
                }
            }
            T::IterationsLimit { limit } => {
                if caused_termination {
                    format!("exceeded iteration limit of {limit}")
                } else {
                    "".to_string()
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use crate::{
        algorithm::search::{Direction, EdgeTraversal, SearchTree},
        model::{
            cost::TraversalCost,
            label::Label,
            network::{EdgeId, EdgeListId, VertexId},
            unit::Cost,
        },
    };

    use super::TerminationModel as T;

    #[test]
    fn test_within_runtime_limit() {
        let within_limit = Duration::from_secs(1);
        let start_time = Instant::now() - within_limit;
        let limit = Duration::from_secs(2);
        let frequency = 10;
        let tree = mock_tree(0);

        let m = T::QueryRuntimeLimit { limit, frequency };

        for iteration in 0..(frequency + 1) {
            let result = m.should_terminate(&start_time, &tree, iteration);
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
        let tree = mock_tree(0);

        let m = T::QueryRuntimeLimit { limit, frequency };

        for iteration in 0..(frequency + 1) {
            let result = m.should_terminate(&start_time, &tree, iteration);
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

        let t4 = mock_tree(4);
        let t5 = mock_tree(5);
        let t6 = mock_tree(6);

        let good = m.should_terminate(&i, &t4, 4);
        let terminate1 = m.should_terminate(&i, &t5, 5);
        let terminate2 = m.should_terminate(&i, &t6, 6);
        assert!(!good);
        assert!(terminate1);
        assert!(terminate2);
    }

    #[test]
    fn test_solution_size_limit() {
        // solution size limit of 5 should accept tree of size 4 + 5, fail on size 6
        let m = T::SolutionSizeLimit { limit: 5 };
        let i = Instant::now();
        let t4 = mock_tree(4);
        let t5 = mock_tree(5);
        let t6 = mock_tree(6);

        let good1 = m.should_terminate(&i, &t4, 4);
        let terminate1 = m.should_terminate(&i, &t5, 5);
        let terminate2 = m.should_terminate(&i, &t6, 6);
        assert!(!good1);
        assert!(terminate1);
        assert!(terminate2);
    }

    #[test]
    fn test_combined_3() {
        let exceeds_limit = Duration::from_secs(3);
        let start_time = Instant::now() - exceeds_limit;
        let runtime_limit = Duration::from_secs(2);
        let frequency = 1;
        let iteration_limit = 5;
        let solution_limit = 3;
        let tree = mock_tree(solution_limit + 1);

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
        let terminate = cm.should_terminate(&start_time, &tree, iteration_limit + 1);
        assert!(terminate);
        let msg = cm.explain(&start_time, &tree, iteration_limit + 1);
        let expected = [
            "exceeded runtime limit of 0:00:02.000",
            "exceeded iteration limit of 5",
            "exceeded solution size limit of 3",
        ]
        .join(", ");
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
        let tree = mock_tree(solution_limit - 1);

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
        let terminate = cm.should_terminate(&start_time, &tree, iteration_limit + 1);
        assert!(terminate);
        let msg = cm.explain(&start_time, &tree, iteration_limit + 1);
        let expected = [
            "exceeded runtime limit of 0:00:02.000",
            "exceeded iteration limit of 5",
        ]
        .join(", ");
        assert_eq!(msg, expected);
    }

    fn mock_tree(size: usize) -> SearchTree {
        let mut tree = SearchTree::new(Direction::Forward);
        if size == 0 {
            return tree;
        }
        // when creating the tree, it will create a root node, so len() will be mock_tree's size + 1
        for idx in 0..(size - 1) {
            let cost = TraversalCost {
                objective_cost: Cost::MIN_COST,
                total_cost: Cost::MIN_COST,
                cost_component: Default::default(),
            };
            let edge_traversal = EdgeTraversal {
                edge_list_id: EdgeListId(0),
                edge_id: EdgeId(idx),
                cost,
                result_state: vec![],
            };
            tree.insert(
                Label::Vertex(VertexId(idx)),
                edge_traversal,
                Label::Vertex(VertexId(idx + 1)),
            )
            .expect("test invariant failed")
        }
        tree
    }
}
