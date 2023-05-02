use im;

use crate::model::graph::{edge_id::EdgeId, vertex_id::VertexId};

type EdgePropertyTable<T> = im::HashMap<EdgeId, T>;

type VertexPropertyTable<T> = im::HashMap<VertexId, T>;

type EdgeEdgePropertyTable<T> = im::HashMap<(EdgeId, EdgeId), T>;
