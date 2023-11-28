use super::json_extensions::UUIDJsonExtensions;
use crate::app::compass::compass_app_error::CompassAppError;
use crate::app::search::search_app_result::SearchAppResult;
use crate::plugin::{output::output_plugin::OutputPlugin, plugin_error::PluginError};
use kdam::Bar;
use kdam::BarExt;

use routee_compass_core::util::fs::{fs_utils, read_utils::read_raw_file};
use std::path::Path;

pub struct UUIDOutputPlugin {
    uuids: Box<[String]>,
}

impl UUIDOutputPlugin {
    pub fn from_file<P: AsRef<Path>>(filename: &P) -> Result<UUIDOutputPlugin, PluginError> {
        let count =
            fs_utils::line_count(filename.clone(), fs_utils::is_gzip(filename)).map_err(|e| {
                PluginError::FileReadError(filename.as_ref().to_path_buf(), e.to_string())
            })?;

        let mut pb = Bar::builder()
            .total(count)
            .animation("fillup")
            .desc("uuid file")
            .build()
            .map_err(PluginError::InternalError)?;

        let cb = Box::new(|| {
            let _ = pb.update(1);
        });

        let uuids = read_raw_file(filename, |_idx, row| Ok(row), Some(cb)).map_err(|e| {
            PluginError::FileReadError(filename.as_ref().to_path_buf(), e.to_string())
        })?;
        println!();
        Ok(UUIDOutputPlugin { uuids })
    }
}

impl OutputPlugin for UUIDOutputPlugin {
    fn process(
        &self,
        output: &serde_json::Value,
        _search_result: &Result<SearchAppResult, CompassAppError>,
    ) -> Result<Vec<serde_json::Value>, PluginError> {
        let mut updated_output = output.clone();
        let (origin_vertex_id, destination_vertex_id) = output.get_od_vertex_ids()?;
        let origin_uuid = self
            .uuids
            .get(origin_vertex_id.0)
            .ok_or(PluginError::UUIDMissing(origin_vertex_id.0))?;
        let destination_uuid = self
            .uuids
            .get(destination_vertex_id.0)
            .ok_or(PluginError::UUIDMissing(destination_vertex_id.0))?;

        updated_output.add_od_uuids(origin_uuid.clone(), destination_uuid.clone())?;

        Ok(vec![updated_output])
    }
}
