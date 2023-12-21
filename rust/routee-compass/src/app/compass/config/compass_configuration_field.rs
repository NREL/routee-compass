use std::fmt::Display;

#[derive(Debug)]
pub enum CompassConfigurationField {
    Graph,
    Frontier,
    Termination,
    Traversal,
    Cost,
    Algorithm,
    Plugins,
    InputPlugins,
    OutputPlugins,
    Parallelism,
    QueryTimeoutMs,
    IncludeTree,
    ChargeDepleting,
    ChargeSustaining,
    SearchOrientation,
}

impl CompassConfigurationField {
    pub fn to_str(&self) -> &'static str {
        match self {
            CompassConfigurationField::Graph => "graph",
            CompassConfigurationField::Traversal => "traversal",
            CompassConfigurationField::Cost => "cost",
            CompassConfigurationField::Frontier => "frontier",
            CompassConfigurationField::Termination => "termination",
            CompassConfigurationField::Algorithm => "algorithm",
            CompassConfigurationField::Parallelism => "parallelism",
            CompassConfigurationField::QueryTimeoutMs => "query_timeout_ms",
            CompassConfigurationField::IncludeTree => "include_tree",
            CompassConfigurationField::Plugins => "plugin",
            CompassConfigurationField::InputPlugins => "input_plugins",
            CompassConfigurationField::OutputPlugins => "output_plugins",
            CompassConfigurationField::ChargeDepleting => "charge_depleting",
            CompassConfigurationField::ChargeSustaining => "charge_sustaining",
            CompassConfigurationField::SearchOrientation => "search_orientation",
        }
    }
}

impl From<CompassConfigurationField> for String {
    fn from(value: CompassConfigurationField) -> Self {
        value.to_string()
    }
}

impl AsRef<str> for CompassConfigurationField {
    fn as_ref(&self) -> &str {
        self.to_str()
    }
}

impl Display for CompassConfigurationField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}
