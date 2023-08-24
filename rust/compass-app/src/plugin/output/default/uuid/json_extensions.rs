use compass_core::model::graph::vertex_id::VertexId;

use crate::plugin::plugin_error::PluginError;

pub enum UUIDJsonField {
    Request,
    OriginVertexId,
    DestinationVertexId,
    OriginVertexUUID,
    DestinationVertexUUID,
}

impl UUIDJsonField {
    pub fn as_str(self) -> &'static str {
        match self {
            UUIDJsonField::Request => "request",
            UUIDJsonField::OriginVertexId => "origin_vertex",
            UUIDJsonField::DestinationVertexId => "destination_vertex",
            UUIDJsonField::OriginVertexUUID => "origin_vertex_uuid",
            UUIDJsonField::DestinationVertexUUID => "destination_vertex_uuid",
        }
    }

    pub fn as_string(self) -> String {
        self.as_str().to_string()
    }
}

pub trait UUIDJsonExtensions {
    fn get_od_vertex_ids(&self) -> Result<(VertexId, VertexId), PluginError>;
    fn add_od_uuids(
        &mut self,
        origin_uuid: String,
        destination_uuid: String,
    ) -> Result<(), PluginError>;
}

impl UUIDJsonExtensions for serde_json::Value {
    fn get_od_vertex_ids(&self) -> Result<(VertexId, VertexId), PluginError> {
        let request = self
            .get(UUIDJsonField::Request.as_str())
            .ok_or(PluginError::MissingField(UUIDJsonField::Request.as_str()))?
            .as_object()
            .ok_or(PluginError::ParseError(
                UUIDJsonField::Request.as_str(),
                "json object",
            ))?;

        let origin_vertex_id = request
            .get(UUIDJsonField::OriginVertexId.as_str())
            .ok_or(PluginError::MissingField(
                UUIDJsonField::OriginVertexId.as_str(),
            ))?
            .as_u64()
            .ok_or(PluginError::ParseError(
                UUIDJsonField::OriginVertexId.as_str(),
                "u64",
            ))?;
        let destination_vertex_id = request
            .get(UUIDJsonField::DestinationVertexId.as_str())
            .ok_or(PluginError::MissingField(
                UUIDJsonField::DestinationVertexId.as_str(),
            ))?
            .as_u64()
            .ok_or(PluginError::ParseError(
                UUIDJsonField::DestinationVertexId.as_str(),
                "u64",
            ))?;
        Ok((
            VertexId(origin_vertex_id as usize),
            VertexId(destination_vertex_id as usize),
        ))
    }
    fn add_od_uuids(
        &mut self,
        origin_uuid: String,
        destination_uuid: String,
    ) -> Result<(), PluginError> {
        let request = self
            .get_mut(UUIDJsonField::Request.as_str())
            .ok_or(PluginError::MissingField(UUIDJsonField::Request.as_str()))?
            .as_object_mut()
            .ok_or(PluginError::ParseError(
                UUIDJsonField::Request.as_str(),
                "json object",
            ))?;

        request.insert(
            UUIDJsonField::OriginVertexUUID.as_string(),
            serde_json::Value::String(origin_uuid),
        );
        request.insert(
            UUIDJsonField::DestinationVertexUUID.as_string(),
            serde_json::Value::String(destination_uuid),
        );
        Ok(())
    }
}
