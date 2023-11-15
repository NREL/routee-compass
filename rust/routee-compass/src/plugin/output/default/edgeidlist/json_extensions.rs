use routee_compass_core::model::road_network::edge_id::EdgeId;

use crate::plugin::plugin_error::PluginError;

pub enum EdgeListField {
    EdgeIdList,
}

impl TryFrom<String> for EdgeListField {
    type Error = PluginError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "edge_list" => Ok(EdgeListField::EdgeIdList),
            _ => Err(PluginError::ParseError(
                String::from("EdgeListField"),
                String::from("String"),
            )),
        }
    }
}

impl EdgeListField {
    fn into_str(self) -> &'static str {
        match self {
            EdgeListField::EdgeIdList => "edge_id_list",
        }
    }

    fn into_string(self) -> String {
        self.into_str().to_string()
    }
}

pub trait EdgeListJsonExtensions {
    fn add_edge_list(&mut self, edge_list: &[EdgeId]) -> Result<(), PluginError>;
    fn get_edge_list(&self) -> Result<Vec<EdgeId>, PluginError>;
}

impl EdgeListJsonExtensions for serde_json::Value {
    fn add_edge_list(&mut self, edge_list: &[EdgeId]) -> Result<(), PluginError> {
        match self {
            serde_json::Value::Object(map) => {
                let edges_json = edge_list.iter().map(|e| serde_json::json!(e.0)).collect();
                map.insert(
                    EdgeListField::EdgeIdList.into_string(),
                    serde_json::Value::Array(edges_json),
                );
                Ok(())
            }
            _ => Err(PluginError::InputError(String::from(
                "OutputResult is not a JSON object",
            ))),
        }
    }

    fn get_edge_list(&self) -> Result<Vec<EdgeId>, PluginError> {
        match self {
            serde_json::Value::Object(map) => {
                let edge_id_list_field = EdgeListField::EdgeIdList.into_str();
                let edge_id_list_json = map
                    .get(edge_id_list_field)
                    .ok_or(PluginError::MissingField(String::from(edge_id_list_field)))?;
                let json_list = edge_id_list_json.as_array().ok_or(PluginError::ParseError(
                    format!("{:?}", edge_id_list_json),
                    String::from("JSON Array"),
                ))?;
                let result: Result<Vec<EdgeId>, PluginError> = json_list
                    .iter()
                    .map(|v| {
                        v.as_u64()
                            .map(|u| EdgeId(u as usize))
                            .ok_or(PluginError::ParseError(
                                format!("{}", v),
                                String::from("u64"),
                            ))
                    })
                    .collect();

                result
            }
            _ => Err(PluginError::InputError(String::from(
                "OutputResult is not a JSON object",
            ))),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_edge_id_list_e2e() {
        let mut json = serde_json::json!({});
        let edge_list = vec![EdgeId(123), EdgeId(456)];
        json.add_edge_list(&edge_list).unwrap();
        let result = json.get_edge_list().unwrap();
        assert_eq!(edge_list, result);
    }
}
