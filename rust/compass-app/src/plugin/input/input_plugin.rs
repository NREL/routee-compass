use crate::plugin::plugin_error::PluginError;

/// Performs some kind of pre-processing on a user query input. The input JSON is available
/// to the plugin as a reference. The plugin produces a vector of zero to many JSON objects that will
/// replace the input JSON.
///
/// A simple No-operation [`InputPlugin`] would simply clone the input and place it in a `Vec`.
///
/// # Default InputPlugins
///
/// The following default set of input plugin builders are found in the [`super::default`] module:
///
/// * [rtree] - map matches x and y coordinates to vertex ids in the network
/// * [grid search] - duplicates a query based on a list of user-defined values
///
/// [rtree]: super::default::rtree::builder::VertexRTreeBuilder
/// [grid search]: super::default::grid_search::builder::GridSearchBuilder
pub trait InputPlugin: Send + Sync {
    /// Applies this [`InputPlugin`] to a user query input, passing along a `Vec` of input
    /// queries as a result which will replace the input.
    ///
    /// # Arguments
    ///
    /// * `input` - the user query input passed to this plugin
    ///
    /// # Returns
    ///
    /// A `Vec` of JSON values to replace the input JSON, or an error
    fn process(&self, input: &serde_json::Value) -> Result<Vec<serde_json::Value>, PluginError>;
}
