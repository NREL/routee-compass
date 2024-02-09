use std::ops::{Deref, DerefMut};

use allocative::Allocative;
use geo::Coord;

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct InternalCoord(pub Coord);

impl Allocative for InternalCoord {
    fn visit<'a, 'b: 'a>(&self, visitor: &'a mut allocative::Visitor<'b>) {
        visitor.enter_self_sized::<Self>().exit();
    }
}

impl Deref for InternalCoord {
    type Target = Coord;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for InternalCoord {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use allocative;
    use geo::coord;

    #[test]
    fn test_visit() {
        let coord = InternalCoord(coord! {x: 1.0, y: 2.0});
        let memory_bytes = allocative::size_of_unique(&coord);
        // should only have two f64s at 8 bytes each
        assert_eq!(memory_bytes, 16);
    }
}
