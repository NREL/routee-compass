use routee_compass_core::model::network::graph::Graph;

use crate::app::compass::config::compass_configuration_field::CompassConfigurationField;

use super::{
    compass_configuration_error::CompassConfigurationError,
    config_json_extension::ConfigJsonExtensions,
};

pub struct DefaultGraphBuilder {}

impl DefaultGraphBuilder {
    /// tries to build a Graph from a JSON object.
    ///
    /// for both edge and vertex lists, we assume all ids can be used as indices
    /// to an array data structure. to find the size of each array, we pass once
    /// through each file to count the number of rows (minus header) of the CSV.
    /// then we can build a Vec *once* and insert rows as we decode them without
    /// a sort.
    ///
    /// # Arguments
    ///
    /// * `params` - configuration JSON object for building a `Graph` instance
    ///
    /// # Returns
    ///
    /// A graph instance, or an error if an IO error occurred.
    pub fn build(params: &serde_json::Value) -> Result<Graph, CompassConfigurationError> {
        let graph_key = CompassConfigurationField::Graph.to_string();
        let edge_list_csv = params.get_config_path(&"edge_list_input_file", &graph_key)?;
        let vertex_list_csv = params.get_config_path(&"vertex_list_input_file", &graph_key)?;

        let graph = Graph::from_files(&edge_list_csv, &vertex_list_csv)?;

        Ok(graph)
    }
}
