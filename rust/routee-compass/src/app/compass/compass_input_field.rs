use std::fmt::Display;

#[derive(Debug)]
pub enum CompassInputField {
    Queries,
    ConfigInputFile,
}

impl CompassInputField {
    pub fn to_str(&self) -> &'static str {
        match self {
            CompassInputField::Queries => "queries",
            CompassInputField::ConfigInputFile => "config_input_file",
        }
    }
}

impl Display for CompassInputField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}
