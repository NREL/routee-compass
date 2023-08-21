use serde::Deserialize;

use crate::plugin::input::rtree::build_rtree_plugin;
use crate::plugin::input::InputPlugin;
use crate::plugin::output::geometry::plugin::build_geometry_plugin_from_file;
use crate::plugin::output::summary::plugin::build_summary_output_plugin;
use crate::plugin::output::uuid::plugin::build_uuid_plugin_from_file;
use crate::plugin::output::OutputPlugin;
use crate::plugin::plugin_error::PluginError;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum InputPluginConfig {
    #[serde(rename = "vertex_rtree")]
    VertexRTree { vertices_file: String },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum OutputPluginConfig {
    #[serde(rename = "summary")]
    Summary,
    #[serde(rename = "geometry")]
    Geometry { edge_file: String },
    #[serde(rename = "uuid")]
    Uuid { uuid_file: String },
}

#[derive(Debug, Deserialize)]
pub struct PluginConfig {
    pub input_plugins: Vec<InputPluginConfig>,
    pub output_plugins: Vec<OutputPluginConfig>,
}

impl TryFrom<&OutputPluginConfig> for OutputPlugin {
    type Error = PluginError;

    fn try_from(conf: &OutputPluginConfig) -> Result<OutputPlugin, Self::Error> {
        match conf {
            OutputPluginConfig::Summary => build_summary_output_plugin(),
            OutputPluginConfig::Geometry { edge_file } => {
                build_geometry_plugin_from_file(&edge_file)
            }
            OutputPluginConfig::Uuid { uuid_file } => build_uuid_plugin_from_file(&uuid_file),
        }
    }
}

impl TryFrom<&InputPluginConfig> for InputPlugin {
    type Error = PluginError;

    fn try_from(conf: &InputPluginConfig) -> Result<InputPlugin, Self::Error> {
        match conf {
            InputPluginConfig::VertexRTree { vertices_file } => build_rtree_plugin(vertices_file),
        }
    }
}
