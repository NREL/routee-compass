use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum CustomInputFormat {
    Dense,
    Sparse,
}
