use std::collections::HashMap;

use crate::model::graphv2::{edge_id::EdgeId, vertex_id::VertexId};

type EdgePropertyTable<T> = HashMap<EdgeId, T>;

type VertexPropertyTable<T> = HashMap<VertexId, T>;

type EdgeEdgePropertyTable<T> = HashMap<(EdgeId, EdgeId), T>;
