use crate::plugin::plugin_error::PluginError;

pub enum TraversalJsonField {
    RouteOutput,
    TreeOutput,
}

impl TraversalJsonField {
    pub fn as_str(self) -> &'static str {
        match self {
            TraversalJsonField::RouteOutput => "route",
            TraversalJsonField::TreeOutput => "tree",
        }
    }

    pub fn to_string(self) -> String {
        self.as_str().to_string()
    }
}

pub trait TraversalJsonExtensions {
    fn get_route_geometry_wkt(&self) -> Result<String, PluginError>;
}

impl TraversalJsonExtensions for serde_json::Value {
    fn get_route_geometry_wkt(&self) -> Result<String, PluginError> {
        let geometry = self
            .get(TraversalJsonField::RouteOutput.as_str())
            .ok_or(PluginError::MissingField(
                TraversalJsonField::RouteOutput.to_string(),
            ))?
            .as_str()
            .ok_or(PluginError::ParseError(
                TraversalJsonField::RouteOutput.to_string(),
                String::from("string"),
            ))?
            .to_string();
        Ok(geometry)
    }
}
