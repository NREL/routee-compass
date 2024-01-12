use serde::Deserialize;

#[derive(Copy, Clone, Deserialize)]
pub struct EdgeHeading {
    pub start_heading: i16,
    pub end_heading: i16,
}

impl EdgeHeading {
    /// Compute the angle between this edge and the next edge
    pub fn next_edge_angle(&self, next_edge_heading: &EdgeHeading) -> i16 {
        (next_edge_heading.start_heading - self.end_heading + 180) % 360 - 180
    }

    /// Compute the angle between this edge and the previous edge
    pub fn previous_edge_angle(&self, previous_edge_heading: &EdgeHeading) -> i16 {
        (self.start_heading - previous_edge_heading.end_heading + 180) % 360 - 180
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_next_edge_angle() {
        let edge_heading = EdgeHeading {
            start_heading: 45,
            end_heading: 90,
        };
        let next_edge_heading = EdgeHeading {
            start_heading: 90,
            end_heading: 135,
        };
        assert_eq!(edge_heading.next_edge_angle(&next_edge_heading), 0);
    }

    #[test]
    fn test_previous_edge_angle() {
        let edge_heading = EdgeHeading {
            start_heading: 45,
            end_heading: 90,
        };
        let previous_edge_heading = EdgeHeading {
            start_heading: 90,
            end_heading: 135,
        };
        assert_eq!(
            edge_heading.previous_edge_angle(&previous_edge_heading),
            -90
        );
    }
}
