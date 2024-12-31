use crate::{algorithm::search::SearchError, model::network::vertex_id::VertexId};

#[derive(Debug, Clone)]
pub struct KspQuery<'a> {
    pub source: VertexId,
    pub target: VertexId,
    pub user_query: &'a serde_json::Value,
    pub k: usize,
}

impl<'a> KspQuery<'a> {
    pub fn new(
        source: VertexId,
        target: VertexId,
        query: &'a serde_json::Value,
        k_default: usize,
    ) -> Result<KspQuery<'a>, SearchError> {
        let k = match query.get("k") {
            Some(k_json) => k_json
                .as_u64()
                .ok_or_else(|| {
                    SearchError::BuildError(format!(
                        "user supplied k value {} is not an integer",
                        k_json
                    ))
                })
                .map(|k_u64| k_u64 as usize),
            None => Ok(k_default),
        }?;
        let ksp_query = KspQuery {
            source,
            target,
            user_query: query,
            k,
        };
        Ok(ksp_query)
    }
}
