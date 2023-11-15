use crate::algorithm::search::search_error::SearchError;
use std::{fmt::Display, str::FromStr};

pub enum SearchAlgorithmType {
    AStar,
}

impl SearchAlgorithmType {
    pub fn to_str(&self) -> &'static str {
        use SearchAlgorithmType as A;
        match self {
            A::AStar => "a*",
        }
    }
}

impl Display for SearchAlgorithmType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl TryFrom<&serde_json::Value> for SearchAlgorithmType {
    type Error = SearchError;

    /// this method takes the configuration object for the search algorithm
    /// and returns the SearchAlgorithmType, expected to be a string at the
    /// key "type" on this object.
    fn try_from(config: &serde_json::Value) -> Result<Self, Self::Error> {
        let type_obj = config
            .get("type")
            .ok_or(SearchError::BuildError(String::from(
                "algorithm config missing 'type' field",
            )))?;
        let alg_string: String = type_obj
            .as_str()
            .ok_or(SearchError::BuildError(format!(
                "'type' must be string, found {:?}",
                type_obj
            )))?
            .into();
        SearchAlgorithmType::from_str(&alg_string)
    }
}

impl FromStr for SearchAlgorithmType {
    type Err = SearchError;

    fn from_str(input: &str) -> Result<SearchAlgorithmType, Self::Err> {
        use SearchAlgorithmType as A;
        match input {
            "a*" | "a star" => Ok(A::AStar),
            _ => Err(SearchError::BuildError(format!(
                "unknown search algorithm {}",
                input
            ))),
        }
    }
}
