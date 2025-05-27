#![doc = include_str!("doc.md")]

pub mod algorithm;
pub mod config;
pub mod model;

pub mod util;

// managing exposure of test assets to only be available when the dev dependency
// feature is active
// #[cfg(feature = "test-utils")]
pub mod testing;
