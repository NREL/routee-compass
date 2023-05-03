use im::Vector;

use crate::model::property::{edge::Edge, vertex::Vertex};

use crate::model::cost::{cost::Cost, cost_error::CostError, metric::Metric};

pub type EdgeMetricFn = dyn Fn(Edge) -> Result<Vector<Metric>, CostError>;
pub type EdgeEdgeMetricFn = dyn Fn((Edge, Edge)) -> Result<Vector<Metric>, CostError>;
pub type CostFn = dyn Fn((Vector<Metric>, Vector<Metric>)) -> Result<Cost, CostError>;
pub type CostEstFn = dyn Fn((Vertex, Vertex)) -> Result<Cost, CostError>;
