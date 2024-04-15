use crate::{
    algorithm::search::{
        search_algorithm::SearchAlgorithm, search_error::SearchError,
        search_instance::SearchInstance, search_result::SearchResult,
    },
    model::road_network::vertex_id::VertexId,
};

pub fn run_ksp(
    _source: VertexId,
    _target: Option<VertexId>,
    _si: &SearchInstance,
    _underlying: Box<SearchAlgorithm>,
) -> Result<Vec<SearchResult>, SearchError> {
    todo!()
}
