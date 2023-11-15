use std::fmt::Display;

use routee_compass_core::model::cost::Cost;

use crate::plugin::plugin_error::PluginError;

pub enum SummaryField {
    Cost,
    Distance,
}

impl TryFrom<String> for SummaryField {
    type Error = PluginError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "cost" => Ok(SummaryField::Cost),
            "distance" => Ok(SummaryField::Distance),
            _ => Err(PluginError::ParseError(
                String::from("SummaryField"),
                String::from("String"),
            )),
        }
    }
}

impl SummaryField {
    fn as_str(&self) -> &'static str {
        match self {
            SummaryField::Cost => "cost",
            SummaryField::Distance => "distance",
        }
    }
}

impl Display for SummaryField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

pub trait SummaryJsonExtensions {
    fn add_cost(&mut self, cost: Cost) -> Result<(), PluginError>;
    fn get_cost(&self) -> Result<Cost, PluginError>;
    fn get_distance(&self) -> Result<f64, PluginError>;
    fn add_distance(&mut self, distance: f64) -> Result<(), PluginError>;
}

impl SummaryJsonExtensions for serde_json::Value {
    fn add_cost(&mut self, cost: Cost) -> Result<(), PluginError> {
        match self {
            serde_json::Value::Object(map) => {
                let cost_f64: f64 = cost.into();
                let cost_json = serde_json::Number::from_f64(cost_f64).ok_or(
                    PluginError::ParseError(String::from("Cost"), String::from("f64")),
                )?;
                map.insert(
                    SummaryField::Cost.to_string(),
                    serde_json::Value::Number(cost_json),
                );
                Ok(())
            }
            _ => Err(PluginError::InputError(String::from(
                "OutputResult is not a JSON object",
            ))),
        }
    }

    fn get_cost(&self) -> Result<Cost, PluginError> {
        match self {
            serde_json::Value::Object(map) => {
                let cost_field = SummaryField::Cost.to_string();
                let cost_json = map
                    .get(&cost_field)
                    .ok_or(PluginError::MissingField(cost_field))?;
                let cost_f64 = cost_json.as_f64().ok_or(PluginError::ParseError(
                    String::from("Cost"),
                    String::from("f64"),
                ))?;
                let cost = Cost::from(cost_f64);
                Ok(cost)
            }
            _ => Err(PluginError::InputError(String::from(
                "OutputResult is not a JSON object",
            ))),
        }
    }

    fn get_distance(&self) -> Result<f64, PluginError> {
        match self {
            serde_json::Value::Object(map) => {
                let distance_field = SummaryField::Distance.to_string();
                let distance_json = map
                    .get(&distance_field)
                    .ok_or(PluginError::MissingField(distance_field))?;
                let distance_f64 = distance_json.as_f64().ok_or(PluginError::ParseError(
                    String::from("Distance"),
                    String::from("f64"),
                ))?;
                Ok(distance_f64)
            }
            _ => Err(PluginError::InputError(String::from(
                "OutputResult is not a JSON object",
            ))),
        }
    }

    fn add_distance(&mut self, distance: f64) -> Result<(), PluginError> {
        match self {
            serde_json::Value::Object(map) => {
                let distance_json = serde_json::Number::from_f64(distance).ok_or(
                    PluginError::ParseError(String::from("Distance"), String::from("f64")),
                )?;
                let json_string = serde_json::Value::Number(distance_json);
                map.insert(SummaryField::Distance.to_string(), json_string);
                Ok(())
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
    fn test_add_cost() {
        let mut json = serde_json::json!({});
        let cost = Cost::from(1.0);
        json.add_cost(cost).unwrap();
        let cost_field: String = SummaryField::Cost.to_string();
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
        let distance_field: String = SummaryField::Distance.to_string();
        let expected_json = serde_json::json!({
            distance_field: 1.0
        });
        assert_eq!(json, expected_json);
    }
}
