use serde::{Deserialize, Serialize};

/// Helper type that can deserialize either a single item or a vector of items
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

impl<T> OneOrMany<T> {
    /// Convert to a vector, regardless of whether it was originally one item or many
    pub fn into_vec(self) -> Vec<T> {
        match self {
            OneOrMany::One(item) => vec![item],
            OneOrMany::Many(items) => items,
        }
    }

    /// Get a reference to the items as a vector
    pub fn as_vec(&self) -> Vec<&T> {
        match self {
            OneOrMany::One(item) => vec![item],
            OneOrMany::Many(items) => items.iter().collect(),
        }
    }

    /// Get the number of items
    pub fn len(&self) -> usize {
        match self {
            OneOrMany::One(_) => 1,
            OneOrMany::Many(items) => items.len(),
        }
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        match self {
            OneOrMany::One(_) => false,
            OneOrMany::Many(items) => items.is_empty(),
        }
    }

    /// Iterate over the items
    pub fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        match self {
            OneOrMany::One(item) => Box::new(std::iter::once(item)),
            OneOrMany::Many(items) => Box::new(items.iter()),
        }
    }
}
