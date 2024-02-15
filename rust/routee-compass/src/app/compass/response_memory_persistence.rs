use serde::{Deserialize, Serialize};

/// declares a policy for search object response memory persistence.
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ResponseMemoryPersistence {
    PersistResponseInMemory,
    DiscardResponseFromMemory,
}
