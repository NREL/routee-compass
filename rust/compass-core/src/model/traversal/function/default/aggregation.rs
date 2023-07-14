use crate::model::traversal::function::function::CostAggregationFunction;
use crate::model::{cost::cost::Cost, traversal::traversal_error::TraversalError};

fn add(costs: &Vec<Cost>) -> Result<Cost, TraversalError> {
    Ok(costs.iter().fold(Cost::ZERO, |acc, c| acc + *c))
}

fn mul(costs: &Vec<Cost>) -> Result<Cost, TraversalError> {
    Ok(costs.iter().fold(Cost::ZERO, |acc, c| Cost(acc.0 * c.0)))
}

pub fn additive_aggregation() -> CostAggregationFunction {
    let f = Box::new(add);
    return f;
}

pub fn multiplicitive_aggregation() -> CostAggregationFunction {
    let f = Box::new(mul);
    return f;
}
