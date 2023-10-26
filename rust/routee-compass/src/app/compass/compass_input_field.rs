use std::fmt::Display;

#[derive(Debug)]
pub enum CompassInputField {
    Queries,
}

impl CompassInputField {
    pub fn to_str(&self) -> &'static str {
        match self {
            CompassInputField::Queries => "queries",
        }
    }
    pub fn to_string(&self) -> String {
        self.to_str().to_string()
    }
}

impl Display for CompassInputField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}
