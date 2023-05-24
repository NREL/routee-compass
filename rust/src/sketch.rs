pub fn network_from_edgelist_csv<S>(path: Path, edge_id_col: &str, ...) -> Result<DirectedGraph, GraphError> {
    todo!()
}

pub fn from_edgelist_csv<S>(path: Path) -> Fn((Vertex, Edge, Vertex, S)) -> Result<Cost, TraversalError> {
    todo!()
}

pub fn from_edgeedgelist_csv<S>(path: Path) -> Fn((Vertex, Edge, Vertex, Edge, Vertex, S)) -> Result<Cost, TraversalError> {
    todo!()
}

pub fn from_zonelist_csv<S>(path: Path, v1_col_name: &str, ...) -> Fn((Vertex, Edge, Vertex, Edge, Vertex, S)) -> Result<Cost, TraversalError> {
    todo!()
}

pub fn init() -> Result<App, AppError> {
    // 1. read config
    // 2. create graph
    // 3. load different lookup tables (optional) and use to build TraversalModel
    // 4. create rtree on vertices
    // 5. ok!
}

pub fn run(app: App, queries: Vec[Queries]) -> Result<Vec[Stuff], AppError> {
    // 1. find start vertices from rtree
    // 2. parallelize queries
    // 3. run searches
    // 4. return serializable result
}