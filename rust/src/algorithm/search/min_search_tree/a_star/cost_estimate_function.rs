use crate::model::property::vertex::Vertex;

use crate::model::cost::{cost::Cost, cost_error::CostError};

pub trait CostEstimateFunction: Sync + Send {
    fn cost(&self, src: Vertex, dst: Vertex) -> Result<Cost, CostError>;
}
