#[derive(thiserror::Error, Debug)]
pub enum MapError {
    #[error("failure matching query to map: {0}")]
    MapMatchError(String),
}
