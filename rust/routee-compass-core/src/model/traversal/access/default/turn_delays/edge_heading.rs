use serde::Deserialize;

#[derive(Copy, Clone, Deserialize)]
pub struct EdgeHeading {
    start_heading: i16,
    end_heading: Option<i16>,
}

impl EdgeHeading {
    pub fn with_start_and_end(start_heading: i16, end_heading: i16) -> Self {
        Self {
            start_heading,
            end_heading: Some(end_heading),
        }
    }

    pub fn with_start(start_heading: i16) -> Self {
        Self {
            start_heading,
            end_heading: None,
        }
    }

    pub fn start_heading(&self) -> i16 {
        self.start_heading
    }
    /// If the end heading is not specified, it is assumed to be the same as the start heading
    pub fn end_heading(&self) -> i16 {
        match self.end_heading {
            Some(end_heading) => end_heading,
            None => self.start_heading,
        }
    }
    /// Compute the angle between this edge and the next edge
    pub fn next_edge_angle(&self, next_edge_heading: &EdgeHeading) -> i16 {
        let angle = next_edge_heading.start_heading() - self.end_heading();
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
    fn test_next_edge_angle() {
        let edge_heading = EdgeHeading::with_start_and_end(45, 90);
        let next_edge_heading = EdgeHeading::with_start_and_end(90, 135);
        assert_eq!(edge_heading.next_edge_angle(&next_edge_heading), 0);
    }

    #[test]
    fn test_next_edge_angle_wrap() {
        let edge_heading = EdgeHeading::with_start_and_end(10, 10);
        let next_edge_heading = EdgeHeading::with_start_and_end(350, 350);
        assert_eq!(edge_heading.next_edge_angle(&next_edge_heading), -20);

        let edge_heading = EdgeHeading::with_start_and_end(350, 350);
        let next_edge_heading = EdgeHeading::with_start_and_end(10, 10);
        assert_eq!(edge_heading.next_edge_angle(&next_edge_heading), 20);
    }
}
