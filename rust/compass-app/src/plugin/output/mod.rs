use compass_core::algorithm::search::{edge_traversal::EdgeTraversal, search_error::SearchError};

use super::plugin_error::PluginError;

pub mod geometry;
pub mod summary;
pub mod uuid;

pub type OutputPlugin = Box<
    dyn Fn(
        &serde_json::Value,
        Result<&Vec<EdgeTraversal>, SearchError>,
    ) -> Result<serde_json::Value, PluginError>,
>;
