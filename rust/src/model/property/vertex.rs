use crate::model::units::ordinate::Ordinate;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Vertex {
    pub x: Ordinate,
    pub y: Ordinate,
}
