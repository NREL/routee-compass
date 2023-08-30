use compass_core::model::cost::cost::Cost;

use crate::plugin::plugin_error::PluginError;

pub enum EdgeListField {
    EdgeList,
}

impl TryFrom<String> for EdgeListField {
    type Error = PluginError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "edge_list" => Ok(EdgeListField::EdgeList),
            _ => Err(PluginError::ParseError("EdgeListField", "String")),
        }
    }
}

impl EdgeListField {
    fn into_str(self) -> &'static str {
        match self {
            EdgeListField::EdgeList => "edge_list",
        }
    }

    fn into_string(self) -> String {
        self.into_str().to_string()
    }
}

pub trait EdgeListJsonExtensions {
    fn add_edge_list(&mut self, edge_list: Vec<EdgeId>) -> Result<(), PluginError>;
    fn edge_list(&self) -> Result<Vec<EdgeId>, PluginError>;
    fn add_distance(&mut self, distance: f64) -> Result<(), PluginError>;
}

impl EdgeListJsonExtensions for serde_json::Value {
    fn add_cost(&mut self, cost: Cost) -> Result<(), PluginError> {
        match self {
            serde_json::Value::Object(map) => {
                let cost_f64: f64 = cost.into();
                let cost_json = serde_json::Number::from_f64(cost_f64)
                    .ok_or(PluginError::ParseError("Cost", "f64"))?;
                map.insert(
                    EdgeListField::Cost.into_string(),
                    serde_json::Value::Number(cost_json),
                );
                Ok(())
            }
            _ => Err(PluginError::InputError("OutputResult is not a JSON object")),
        }
    }

    fn get_cost(&self) -> Result<Cost, PluginError> {
        match self {
            serde_json::Value::Object(map) => {
                let cost_field = EdgeListField::Cost.into_str();
                let cost_json = map
                    .get(cost_field)
                    .ok_or(PluginError::MissingField(cost_field))?;
                let cost_f64 = cost_json
                    .as_f64()
                    .ok_or(PluginError::ParseError("Cost", "f64"))?;
                let cost = Cost::from(cost_f64);
                Ok(cost)
            }
            _ => Err(PluginError::InputError("OutputResult is not a JSON object")),
        }
    }

    fn get_distance(&self) -> Result<f64, PluginError> {
        match self {
            serde_json::Value::Object(map) => {
                let distance_field = EdgeListField::Distance.into_str();
                let distance_json = map
                    .get(distance_field)
                    .ok_or(PluginError::MissingField(distance_field))?;
                let distance_f64 = distance_json
                    .as_f64()
                    .ok_or(PluginError::ParseError("Distance", "f64"))?;
                Ok(distance_f64)
            }
            _ => Err(PluginError::InputError("OutputResult is not a JSON object")),
        }
    }

    fn add_distance(&mut self, distance: f64) -> Result<(), PluginError> {
        match self {
            serde_json::Value::Object(map) => {
                let distance_json = serde_json::Number::from_f64(distance)
                    .ok_or(PluginError::ParseError("Distance", "f64"))?;
                let json_string = serde_json::Value::Number(distance_json);
                map.insert(EdgeListField::Distance.into_string(), json_string);
                Ok(())
            }
            _ => Err(PluginError::InputError("OutputResult is not a JSON object")),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_add_cost() {
        let mut json = serde_json::json!({});
        let cost = Cost::from(1.0);
        json.add_cost(cost).unwrap();
        let cost_field: String = EdgeListField::Cost.into_string();
        let expected_json = serde_json::json!({
            cost_field: 1.0
        });
        assert_eq!(json, expected_json);
    }

    #[test]
    fn test_add_distance() {
        let mut json = serde_json::json!({});
        let distance = 1.0;
        json.add_distance(distance).unwrap();
        let distance_field: String = EdgeListField::Distance.into_string();
        let expected_json = serde_json::json!({
            distance_field: 1.0
        });
        assert_eq!(json, expected_json);
    }
}
