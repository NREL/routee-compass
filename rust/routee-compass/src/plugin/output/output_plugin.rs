use crate::app::compass::compass_app_error::CompassAppError;
use crate::app::search::search_app_result::SearchAppResult;
use crate::plugin::plugin_error::PluginError;

/// Performs some kind of post-processing on a search result. The result JSON is available
/// to the plugin as a reference which was potentially modified upstream by another output
/// plugin. The plugin will output a modified version of the JSON as a result.
///
/// A simple No-operation [`OutputPlugin`] would simply clone its JSON argument.
///
/// # Default OutputPlugins
///
/// The following default set of output plugin builders are found in the [`super::default`] module:
///
/// * [summary] - simple plugin appends cost and distance to result
/// * [traversal] - fully-featured plugin for traversal outputs in different formats
/// * [uuid] - attach the original graph ids to a result
///
/// [summary]: super::default::summary::builder::SummaryOutputPluginBuilder
/// [traversal]: super::default::traversal::builder::TraversalPluginBuilder
/// [uuid]: super::default::uuid::builder::UUIDOutputPluginBuilder
pub trait OutputPlugin: Send + Sync {
    /// Applies this [`OutputPlugin`] to a search result, passing along a JSON
    /// that will replace the `output` JSON argument.
    ///
    /// # Arguments
    ///
    /// * `output` - the search result passed to this plugin
    /// * `result` - the result of the search via the [internal representation].
    ///              this is passed as a `Result` as the search may have failed.
    ///
    /// # Returns
    ///
    /// The modified JSON or an error
    ///
    /// [internal representation]: crate::app::search::search_app_result::SearchAppResult
    fn process(
        &self,
        output: &serde_json::Value,
        result: &Result<SearchAppResult, CompassAppError>,
    ) -> Result<Vec<serde_json::Value>, PluginError>;
}
