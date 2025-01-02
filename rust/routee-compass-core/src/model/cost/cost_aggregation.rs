use crate::model::unit::{AsF64, Cost};
use serde::{Deserialize, Serialize};

use super::cost_model_error::CostModelError;

#[derive(Deserialize, Serialize, Clone, Copy, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub enum CostAggregation {
    #[default]
    Sum,
    Mul,
}

impl CostAggregation {
    pub fn agg(&self, costs: &[(&String, Cost)]) -> Cost {
        match self {
            CostAggregation::Sum => costs.iter().fold(Cost::ZERO, |acc, (_, c)| acc + *c),
            CostAggregation::Mul => {
                if costs.is_empty() {
                    Cost::ZERO
                } else {
                    costs.iter().fold(Cost::ONE, |acc, (_, c)| {
                        Cost::new(acc.as_f64() * c.as_f64())
                    })
                }
            }
        }
    }

    pub fn agg_iter<'a>(
        &self,
        costs: impl Iterator<Item = Result<(&'a String, Cost), CostModelError>>,
    ) -> Result<Cost, CostModelError> {
        match self {
            CostAggregation::Sum => {
                let mut sum = Cost::ZERO;
                for cost in costs {
                    let (_, cost) = cost?;
                    sum = sum + cost;
                }
                Ok(sum)
            }
            CostAggregation::Mul => {
                // test if the iterator is empty
                let mut costs = costs.peekable();
                if costs.peek().is_none() {
                    return Ok(Cost::ZERO);
                }
                let mut product = Cost::ONE;
                for cost in costs {
                    let (_, cost) = cost?;
                    product = Cost::new(product.as_f64() * cost.as_f64());
                }
                Ok(product)
            }
        }
    }
}
