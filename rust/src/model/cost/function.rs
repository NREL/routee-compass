use im::Vector;

use crate::model::property::{edge::Edge, vertex::Vertex};

use super::{cost::Cost, metric::Metric};

#[derive(thiserror::Error, Debug, Clone)]
pub enum CostError {
    #[error("vector of metric observations is wrong length for provided cost function")]
    MetricVectorSizeMismatch,
}

pub type EdgeMetricFn = dyn Fn(Edge) -> Result<Vector<Metric>, CostError>;
pub type EdgeEdgeMetricFn = dyn Fn((Edge, Edge)) -> Result<Vector<Metric>, CostError>;
pub type CostFn = dyn Fn((Vector<Metric>, Vector<Metric>)) -> Result<Cost, CostError>;
pub type CostEstFn = dyn Fn((Vertex, Vertex)) -> Result<Cost, CostError>;
