use serde::{Deserialize, Serialize};

/// declares a policy for search object response memory persistence.
#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ResponsePersistencePolicy {
    PersistResponseInMemory,
    DiscardResponseFromMemory,
}
