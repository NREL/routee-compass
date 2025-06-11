use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum WeightUnit {
    Pounds,
    Tons,
    Kg,
}

impl std::fmt::Display for WeightUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_string(self)
            .map_err(|_| std::fmt::Error)?
            .replace('\"', "");
        write!(f, "{}", s)
    }
}

impl FromStr for WeightUnit {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pound" | "pounds" | "lb" | "lbs" => Ok(Self::Pounds),
            "ton" | "tons" => Ok(Self::Tons),
            "kilogram" | "kilograms" | "kg" | "kgs" => Ok(Self::Kg),
            _ => Err(format!("unknown weight unit '{}'", s)),
        }
    }
}

impl TryFrom<String> for WeightUnit {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}
