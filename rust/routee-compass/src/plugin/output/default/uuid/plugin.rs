use super::json_extensions::UUIDJsonExtensions;
use crate::app::compass::compass_app_error::CompassAppError;
use crate::app::search::search_app_result::SearchAppResult;
use crate::plugin::output::default::uuid::json_extensions::UUIDJsonField;
use crate::plugin::{output::output_plugin::OutputPlugin, plugin_error::PluginError};
use kdam::Bar;
use kdam::BarExt;

use routee_compass_core::algorithm::search::search_instance::SearchInstance;
use routee_compass_core::util::fs::{fs_utils, read_utils::read_raw_file};
use std::path::Path;

pub struct UUIDOutputPlugin {
    uuids: Box<[String]>,
    o_key: String,
    d_key: String,
}

impl UUIDOutputPlugin {
    pub fn from_file<P: AsRef<Path>>(filename: &P) -> Result<UUIDOutputPlugin, PluginError> {
        let count = fs_utils::line_count(filename, fs_utils::is_gzip(filename)).map_err(|e| {
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

        let o_key = UUIDJsonField::OriginVertexUUID.to_string();
        let d_key = UUIDJsonField::DestinationVertexUUID.to_string();
        Ok(UUIDOutputPlugin {
            uuids,
            o_key,
            d_key,
        })
    }
}

impl OutputPlugin for UUIDOutputPlugin {
    fn process(
        &self,
        output: &mut serde_json::Value,
        search_result: &Result<(SearchAppResult, SearchInstance), CompassAppError>,
    ) -> Result<(), PluginError> {
        match search_result {
            Err(_) => Ok(()),
            Ok(_) => {
                let (origin_vertex_id, destination_vertex_id) = output.get_od_vertex_ids()?;
                let origin_uuid = self
                    .uuids
                    .get(origin_vertex_id.0)
                    .cloned()
                    .ok_or_else(|| PluginError::UUIDMissing(origin_vertex_id.0))?;
                let destination_uuid = self
                    .uuids
                    .get(destination_vertex_id.0)
                    .cloned()
                    .ok_or_else(|| PluginError::UUIDMissing(destination_vertex_id.0))?;

                output[&self.o_key] = serde_json::Value::String(origin_uuid);
                output[&self.d_key] = serde_json::Value::String(destination_uuid);
                Ok(())
            }
        }
    }
}
