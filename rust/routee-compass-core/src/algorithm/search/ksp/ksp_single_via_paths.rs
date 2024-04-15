use crate::{
    algorithm::search::{
        search_algorithm::SearchAlgorithm, search_error::SearchError,
        search_instance::SearchInstance, search_result::SearchResult,
    },
    model::road_network::vertex_id::VertexId,
};

pub fn run_ksp(
    source: VertexId,
    target: Option<VertexId>,
    si: &SearchInstance,
    underlying: Box<SearchAlgorithm>,
) -> Result<Vec<SearchResult>, SearchError> {
    todo!()
}
