use super::json_extensions::UUIDJsonExtensions;

use compass_core::algorithm::search::edge_traversal::EdgeTraversal;
use compass_core::algorithm::search::search_error::SearchError;
use compass_core::util::fs::{fs_utils, read_utils::read_raw_file};
use kdam::Bar;
use kdam::BarExt;

use crate::plugin::{output::OutputPlugin, plugin_error::PluginError};

pub fn build_uuid_plugin_from_file(filename: &String) -> Result<OutputPlugin, PluginError> {
    let count = fs_utils::line_count(filename.clone(), fs_utils::is_gzip(&filename))
        .map_err(PluginError::FileReadError)?;

    let mut pb = Bar::builder()
        .total(count)
        .animation("fillup")
        .desc("uuid file")
        .build()
        .map_err(PluginError::InternalError)?;

    let cb = Box::new(|| {
        pb.update(1);
    });

    let all_uuids = read_raw_file(&filename, |_idx, row| Ok(row), Some(cb))?;
    let uuid_lookup_fn = move |output: &serde_json::Value,
                               _search_result: Result<&Vec<EdgeTraversal>, SearchError>|
          -> Result<serde_json::Value, PluginError> {
        let mut updated_output = output.clone();
        let (origin_vertex_id, destination_vertex_id) = output.get_od_vertex_ids()?;
        let origin_uuid = all_uuids
            .get(origin_vertex_id.0 as usize)
            .ok_or(PluginError::UUIDMissing(origin_vertex_id.0))?;
        let destination_uuid = all_uuids
            .get(destination_vertex_id.0 as usize)
            .ok_or(PluginError::UUIDMissing(destination_vertex_id.0))?;

        updated_output.add_od_uuids(origin_uuid.clone(), destination_uuid.clone())?;

        Ok(updated_output)
    };
    Ok(Box::new(uuid_lookup_fn))
}
