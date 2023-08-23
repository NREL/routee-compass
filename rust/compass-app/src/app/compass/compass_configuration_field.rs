use std::fmt::Display;

#[derive(Debug)]
pub enum CompassConfigurationField {
    Graph,
    Frontier,
    Traversal,
    Algorithm,
    Plugins,
    InputPlugins,
    OutputPlugins,
}

impl CompassConfigurationField {
    pub fn to_str(&self) -> &'static str {
        match self {
            CompassConfigurationField::Graph => "graph",
            CompassConfigurationField::Traversal => "traversal",
            CompassConfigurationField::Frontier => "frontier",
            CompassConfigurationField::Algorithm => "algorithm",
            CompassConfigurationField::Plugins => "plugins",
            CompassConfigurationField::InputPlugins => "input_plugins",
            CompassConfigurationField::OutputPlugins => "output_plugins",
        }
    }
    pub fn to_string(&self) -> String {
        self.to_str().to_string()
    }
}

impl Display for CompassConfigurationField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}
