use super::CompassAppError;
use crate::plugin::{input::InputJsonExtensions, PluginError};
use kdam::{Bar, BarExt};
use ordered_float::OrderedFloat;
use routee_compass_core::util::progress;

/// applies the weight balancing policy set by the LoadBalancerPlugin InputPlugin.
///
/// # Arguments
///
/// * `queries` - user queries to load balance based on a query weight heuristic.
/// * `parallelism` - number of chunks to split inputs into, set by user
/// * `default` - weight value if weight heuristic fails to produce an estimate
///
/// # Returns
///
/// An index for sorting the values so that, when fed into rayon's par_chunks iterator,
/// load balances the queries across processes based on the estimates. the resulting
/// batches are not equal-sized
pub fn apply_load_balancing_policy(
    queries: Vec<serde_json::Value>,
    parallelism: usize,
    default: f64,
) -> Result<Vec<Vec<serde_json::Value>>, CompassAppError> {
    if queries.is_empty() {
        return Ok(vec![]);
    }

    let mut bin_totals = vec![0.0; parallelism];
    let mut assignments: Vec<Vec<serde_json::Value>> = vec![vec![]; parallelism];
    let n_queries = queries.len();

    let bar_builder = Bar::builder()
        .total(n_queries)
        .desc("load balancing")
        .animation("fillup");
    let mut bar_opt = progress::build_progress_bar(bar_builder);
    for q in queries.into_iter() {
        let w = q.get_query_weight_estimate()?.unwrap_or(default);
        let min_bin = min_bin(&bin_totals)?;
        bin_totals[min_bin] += w;
        assignments[min_bin].push(q);
        if let Some(ref mut bar) = bar_opt {
            let _ = bar.update(1);
        }
    }
    Ok(assignments)
}

fn min_bin(bins: &[f64]) -> Result<usize, PluginError> {
    bins.iter()
        .enumerate()
        .min_by_key(|(_i, w)| OrderedFloat(**w))
        .map(|(i, _w)| i)
        .ok_or_else(|| {
            PluginError::InternalError(String::from("cannot find min bin of empty slice"))
        })
}

#[cfg(test)]
mod test {
    use super::apply_load_balancing_policy;
    use crate::plugin::input::InputField;
    use serde_json::json;

    fn test_run_policy(queries: Vec<serde_json::Value>, parallelism: usize) -> Vec<Vec<i64>> {
        apply_load_balancing_policy(queries, parallelism, 1.0)
            .unwrap()
            .iter()
            .map(|qs| {
                let is: Vec<i64> = qs
                    .iter()
                    .map(|q| q.get("index").unwrap().as_i64().unwrap())
                    .collect();
                is
            })
            .collect::<Vec<_>>()
    }

    #[test]
    fn test_uniform_input() {
        // striped
        let queries: Vec<serde_json::Value> = (0..12)
            .map(|i| {
                json!({
                    "index": i,
                    InputField::QueryWeightEstimate.to_str(): 1
                })
            })
            .collect();
        let parallelism = 4;
        let result = test_run_policy(queries, parallelism);
        let expected: Vec<Vec<i64>> =
            vec![vec![0, 4, 8], vec![1, 5, 9], vec![2, 6, 10], vec![3, 7, 11]];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_incremental_input() {
        // this produces the same layout as the uniform input
        let queries: Vec<serde_json::Value> = (0..12)
            .map(|i| {
                json!({
                    "index": i,
                    InputField::QueryWeightEstimate.to_str(): i + 1
                })
            })
            .collect();
        let parallelism = 4;
        let result = test_run_policy(queries, parallelism);
        let expected: Vec<Vec<i64>> =
            vec![vec![0, 4, 8], vec![1, 5, 9], vec![2, 6, 10], vec![3, 7, 11]];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_cycling_input() {
        // an input one can verify via debugging that produces the expected output below
        let queries: Vec<serde_json::Value> = [1, 4, 1, 2, 1, 4, 1, 2, 1, 4, 1, 2]
            .iter()
            .enumerate()
            .map(|(i, estimate)| {
                json!({
                    "index": i,
                    InputField::QueryWeightEstimate.to_str(): estimate
                })
            })
            .collect();
        let parallelism = 4;
        let result = test_run_policy(queries, parallelism);
        let expected = vec![vec![0, 4, 6, 8, 9], vec![1, 10], vec![2, 5], vec![3, 7, 11]];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_big_outlier() {
        let queries: Vec<serde_json::Value> = [4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]
            .iter()
            .enumerate()
            .map(|(idx, est)| {
                json!({
                    "index": idx,
                    InputField::QueryWeightEstimate.to_str(): est
                })
            })
            .collect();
        let parallelism = 4;
        let result = test_run_policy(queries, parallelism);
        let expected = vec![vec![0], vec![1, 4, 7, 10], vec![2, 5, 8, 11], vec![3, 6, 9]];
        assert_eq!(result, expected);
    }
}
