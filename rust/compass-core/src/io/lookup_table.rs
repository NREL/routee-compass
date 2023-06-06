use crate::model::cost::cost::Cost;
use crate::model::property::{edge::Edge, vertex::Vertex};
use crate::model::traversal::traversal_error::TraversalError;

pub type EdgeLookupTable<S> =
    Box<dyn Fn((Vertex, Edge, Vertex, S)) -> Result<Cost, TraversalError>>;

pub type EdgeEdgeLookupTable<S> =
    Box<dyn Fn((Vertex, Edge, Vertex, Edge, Vertex, S)) -> Result<Cost, TraversalError>>;
