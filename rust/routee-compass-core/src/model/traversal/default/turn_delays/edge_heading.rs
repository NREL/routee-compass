use serde::Deserialize;

/// simplifies the representation of directionality for a linestring
/// to just the headings of the start and end points, using cardinal angles [0, 360).
/// if the start and end have the same heading, the edge heading is None.
#[derive(Copy, Clone, Deserialize)]
pub struct EdgeHeading {
    arrival_heading: i16,
    departure_heading: Option<i16>,
}

impl EdgeHeading {
    /// creates an EdgeHeading from a start and end heading
    pub fn new(arrival_heading: i16, departure_heading: i16) -> Self {
        Self {
            arrival_heading,
            departure_heading: Some(departure_heading),
        }
    }

    /// retrieve the start
    pub fn start_heading(&self) -> i16 {
        self.arrival_heading
    }

    /// If the end heading is not specified, it is assumed to be the same as the start heading
    pub fn end_heading(&self) -> i16 {
        match self.departure_heading {
            Some(end_heading) => end_heading,
            None => self.arrival_heading,
        }
    }
    /// Compute the angle between this edge and some destination edge.
    pub fn bearing_to_destination(&self, destination: &EdgeHeading) -> i16 {
        let angle = destination.start_heading() - self.end_heading();
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
