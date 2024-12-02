use std::fmt::Display;

use routee_compass_core::model::network::vertex_id::VertexId;

use crate::plugin::{input::InputField, output::OutputPluginError};

pub enum UUIDJsonField {
    Request,
    OriginVertexId,
    DestinationVertexId,
    OriginVertexUUID,
    DestinationVertexUUID,
}

impl From<UUIDJsonField> for InputField {
    fn from(value: UUIDJsonField) -> Self {
        InputField::Custom(value.to_string())
    }
}

impl UUIDJsonField {
    pub fn as_str(&self) -> &'static str {
        match self {
            UUIDJsonField::Request => "request",
            UUIDJsonField::OriginVertexId => "origin_vertex",
            UUIDJsonField::DestinationVertexId => "destination_vertex",
            UUIDJsonField::OriginVertexUUID => "origin_vertex_uuid",
            UUIDJsonField::DestinationVertexUUID => "destination_vertex_uuid",
        }
    }
}

impl Display for UUIDJsonField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

pub trait UUIDJsonExtensions {
    fn get_od_vertex_ids(&self) -> Result<(VertexId, VertexId), OutputPluginError>;
    fn add_od_uuids(
        &mut self,
        origin_uuid: String,
        destination_uuid: String,
    ) -> Result<(), OutputPluginError>;
}

impl UUIDJsonExtensions for serde_json::Value {
    fn get_od_vertex_ids(&self) -> Result<(VertexId, VertexId), OutputPluginError> {
        let request = self
            .get(UUIDJsonField::Request.as_str())
            .ok_or_else(|| {
                OutputPluginError::MissingExpectedQueryField(UUIDJsonField::Request.into())
            })?
            .as_object()
            .ok_or_else(|| {
                OutputPluginError::QueryFieldHasInvalidType(
                    UUIDJsonField::Request.into(),
                    String::from("json object"),
                )
            })?;

        let origin_vertex_id = request
            .get(&UUIDJsonField::OriginVertexId.to_string())
            .ok_or_else(|| {
                OutputPluginError::MissingExpectedQueryField(UUIDJsonField::OriginVertexId.into())
            })?
            .as_u64()
            .ok_or_else(|| {
                OutputPluginError::QueryFieldHasInvalidType(
                    UUIDJsonField::OriginVertexId.into(),
                    String::from("u64"),
                )
            })?;
        let destination_vertex_id = request
            .get(&UUIDJsonField::DestinationVertexId.to_string())
            .ok_or_else(|| {
                OutputPluginError::MissingExpectedQueryField(
                    UUIDJsonField::DestinationVertexId.into(),
                )
            })?
            .as_u64()
            .ok_or_else(|| {
                OutputPluginError::QueryFieldHasInvalidType(
                    UUIDJsonField::DestinationVertexId.into(),
                    String::from("u64"),
                )
            })?;
        Ok((
            VertexId(origin_vertex_id as usize),
            VertexId(destination_vertex_id as usize),
        ))
    }
    fn add_od_uuids(
        &mut self,
        origin_uuid: String,
        destination_uuid: String,
    ) -> Result<(), OutputPluginError> {
        let request = self
            .get_mut(UUIDJsonField::Request.as_str())
            .ok_or_else(|| {
                OutputPluginError::MissingExpectedQueryField(UUIDJsonField::Request.into())
            })?
            .as_object_mut()
            .ok_or_else(|| {
                OutputPluginError::QueryFieldHasInvalidType(
                    UUIDJsonField::Request.into(),
                    String::from("json object"),
                )
            })?;

        request.insert(
            UUIDJsonField::OriginVertexUUID.to_string(),
            serde_json::Value::String(origin_uuid),
        );
        request.insert(
            UUIDJsonField::DestinationVertexUUID.to_string(),
            serde_json::Value::String(destination_uuid),
        );
        Ok(())
    }
}
