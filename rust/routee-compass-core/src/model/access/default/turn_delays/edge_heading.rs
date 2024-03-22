use serde::Deserialize;

/// simplifies the representation of directionality for a linestring
/// to just the headings of the start and end points, using cardinal angles [0, 360).
/// if the start and end have the same heading, the edge heading is None.
#[derive(Copy, Clone, Deserialize)]
pub struct EdgeHeading {
    start_angle: i16,
    end_angle: Option<i16>,
}

impl EdgeHeading {
    /// creates an EdgeHeading from a start and end heading
    pub fn new(start_angle: i16, end_angle: i16) -> Self {
        Self {
            start_angle,
            end_angle: Some(end_angle),
        }
    }

    /// retrieve the start
    pub fn start_angle(&self) -> i16 {
        self.start_angle
    }

    /// If the end heading is not specified, it is assumed to be the same as the start heading
    pub fn end_angle(&self) -> i16 {
        match self.end_angle {
            Some(end_heading) => end_heading,
            None => self.start_angle,
        }
    }
    /// Compute the angle between this edge and some destination edge.
    pub fn bearing_to_destination(&self, destination: &EdgeHeading) -> i16 {
        let angle = destination.start_angle() - self.end_angle();
        if angle > 180 {
            angle - 360
        } else if angle < -180 {
            angle + 360
        } else {
            angle
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_simple() {
        let edge_heading = EdgeHeading::new(45, 90);
        let next_edge_heading = EdgeHeading::new(90, 135);
        assert_eq!(edge_heading.bearing_to_destination(&next_edge_heading), 0);
    }

    #[test]
    fn test_wrap_360() {
        let edge_heading = EdgeHeading::new(10, 10);
        let next_edge_heading = EdgeHeading::new(350, 350);
        assert_eq!(edge_heading.bearing_to_destination(&next_edge_heading), -20);

        let edge_heading = EdgeHeading::new(350, 350);
        let next_edge_heading = EdgeHeading::new(10, 10);
        assert_eq!(edge_heading.bearing_to_destination(&next_edge_heading), 20);
    }
}
