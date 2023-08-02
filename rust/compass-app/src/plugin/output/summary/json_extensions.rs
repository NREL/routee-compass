use compass_core::model::cost::cost::Cost;

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
            _ => Err(PluginError::ParseError("SummaryField", "String")),
        }
    }
}

impl Into<String> for SummaryField {
    fn into(self) -> String {
        match self {
            SummaryField::Cost => String::from("cost"),
            SummaryField::Distance => String::from("distance"),
        }
    }
}

pub trait SummaryJsonExtensions {
    fn add_cost(&mut self, cost: Cost) -> Result<(), PluginError>;
    fn add_distance(&mut self, distance: f64) -> Result<(), PluginError>;
}

impl SummaryJsonExtensions for serde_json::Value {
    fn add_cost(&mut self, cost: Cost) -> Result<(), PluginError> {
        match self {
            serde_json::Value::Object(map) => {
                let cost_f64: f64 = cost.into();
                let cost_json = serde_json::Number::from_f64(cost_f64)
                    .ok_or(PluginError::ParseError("Cost", "f64"))?;
                map.insert(
                    SummaryField::Cost.into(),
                    serde_json::Value::Number(cost_json),
                );
                Ok(())
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
                map.insert(SummaryField::Distance.into(), json_string);
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
        let cost_field: String = SummaryField::Cost.into();
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
        let distance_field: String = SummaryField::Distance.into();
        let expected_json = serde_json::json!({
            distance_field: 1.0
        });
        assert_eq!(json, expected_json);
    }
}
