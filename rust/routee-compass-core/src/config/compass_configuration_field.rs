use std::fmt::Display;

#[derive(Debug)]
pub enum CompassConfigurationField {
    Graph,
    Constraint,
    Termination,
    Label,
    State,
    Traversal,
    Access,
    Cost,
    Algorithm,
    Plugins,
    MapModel,
    InputPlugins,
    OutputPlugins,
    Parallelism,
    QueryTimeoutMs,
    IncludeTree,
    ChargeDepleting,
    ChargeSustaining,
    ResponsePersistencePolicy,
    ResponseOutputPolicy,
}

impl CompassConfigurationField {
    pub fn to_str(&self) -> &'static str {
        match self {
            CompassConfigurationField::Graph => "graph",
            CompassConfigurationField::Traversal => "traversal",
            CompassConfigurationField::Access => "access",
            CompassConfigurationField::Cost => "cost",
            CompassConfigurationField::State => "state",
            CompassConfigurationField::Constraint => "constraint",
            CompassConfigurationField::Termination => "termination",
            CompassConfigurationField::Algorithm => "algorithm",
            CompassConfigurationField::Parallelism => "parallelism",
            CompassConfigurationField::QueryTimeoutMs => "query_timeout_ms",
            CompassConfigurationField::IncludeTree => "include_tree",
            CompassConfigurationField::Plugins => "plugin",
            CompassConfigurationField::MapModel => "mapping",
            CompassConfigurationField::InputPlugins => "input_plugins",
            CompassConfigurationField::OutputPlugins => "output_plugins",
            CompassConfigurationField::ChargeDepleting => "charge_depleting",
            CompassConfigurationField::ChargeSustaining => "charge_sustaining",
            CompassConfigurationField::ResponsePersistencePolicy => "response_persistence_policy",
            CompassConfigurationField::ResponseOutputPolicy => "response_output_policy",
            CompassConfigurationField::Label => "label",
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
