#[derive(thiserror::Error, Debug, Clone)]
pub enum RoadMapError{
    #[error("No path from {origin:?} to {destination:?}")]
    NoPath { origin: [isize; 2], destination: [isize; 2] },
}
