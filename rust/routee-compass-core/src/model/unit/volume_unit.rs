use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Default)]
#[serde(rename_all = "snake_case", try_from = "String")]
pub enum VolumeUnit {
    #[default]
    GallonsUs,
    GallonsUk,
    Liters,
}

impl std::fmt::Display for VolumeUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_string(self)
            .map_err(|_| std::fmt::Error)?
            .replace('\"', "");
        write!(f, "{}", s)
    }
}

impl FromStr for VolumeUnit {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use VolumeUnit as V;
        match s.trim().to_lowercase().as_str() {
            "gal" | "usgal" | "usgals" => Ok(V::GallonsUs),
            "ukgal" | "ukgals" => Ok(V::GallonsUk),
            "liter" | "liters" | "l" => Ok(V::Liters),
            _ => Err(format!("unknown volume unit '{}'", s)),
        }
    }
}

impl TryFrom<String> for VolumeUnit {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}
