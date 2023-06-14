struct TomTomEdgeList {}

use super::tomtom_graph_error::TomTomGraphError;

impl TryFrom<String> for TomTomEdgeList {
    type Error = TomTomGraphError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        todo!("copy edge list code from tomtom_graph.rs")
    }
}
