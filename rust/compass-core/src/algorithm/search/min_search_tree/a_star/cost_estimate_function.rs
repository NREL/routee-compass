use crate::model::property::vertex::Vertex;

use crate::model::cost::{cost::Cost, cost_error::CostError};
use crate::model::units::milliseconds::Milliseconds;

pub trait CostEstimateFunction: Sync + Send {
    fn cost(&self, src: Vertex, dst: Vertex) -> Result<Cost, CostError>;
}

/// the default cost estimate function uses the haversine formula to give us
/// the lower bound on distance between two vertices. we then "traverse" that
/// distance using the provided travel speed in kilometers per hour.
pub struct Haversine {
    pub travel_speed_kph: f64,
}

impl CostEstimateFunction for Haversine {
    /// uses the haversine distance between two points to estimate a lower bound of
    /// travel time in milliseconds.
    fn cost(&self, src: Vertex, dst: Vertex) -> Result<Cost, CostError> {
        let distance_km = Haversine::distance_km(src.x.0, src.y.0, dst.x.0, dst.y.0);
        let travel_time_hours = distance_km / self.travel_speed_kph;
        let travel_time_ms = Milliseconds::from_hours(travel_time_hours);
        Ok(Cost(travel_time_ms.0))
    }
}

impl Haversine {
    pub const APPROX_EARTH_RADIUS_KM: f64 = 6372.8;

    /// haversine distance formula, based on the one published to rosetta code.
    /// https://rosettacode.org/wiki/Haversine_formula#Rust
    /// computes the great circle distance between two points in kilometers.
    pub fn distance_km(src_x: f64, src_y: f64, dst_x: f64, dst_y: f64) -> f64 {
        let lat1 = src_y.to_radians();
        let lat2 = dst_y.to_radians();
        let d_lat = lat2 - lat1;
        let d_lon = (dst_x - src_x).to_radians();

        let a = (d_lat / 2.0).sin().powi(2) + (d_lon / 2.0).sin().powi(2) * lat1.cos() * lat2.cos();
        let c = 2.0 * a.sqrt().asin();
        let distance_km = Haversine::APPROX_EARTH_RADIUS_KM * c;
        distance_km
    }
}

#[cfg(test)]
mod tests {
    use crate::model::units::milliseconds::Milliseconds;

    use crate::model::{
        graph::vertex_id::VertexId, property::vertex::Vertex, units::ordinate::Ordinate,
    };

    use super::{CostEstimateFunction, Haversine};

    #[test]
    fn test_haversine() {
        // based on test case at the rosetta code website:
        // https://rosettacode.org/wiki/Haversine_formula#Rust

        let h = Haversine {
            travel_speed_kph: 40.0,
        };
        let src = Vertex {
            vertex_id: VertexId(0),
            x: Ordinate(-86.67),
            y: Ordinate(36.12),
        };
        let dst = Vertex {
            vertex_id: VertexId(1),
            x: Ordinate(-118.4),
            y: Ordinate(33.94),
        };

        let expected_dist = 2887.2599506071106;
        let expected_time_hours = expected_dist / 40.0;
        let expected_time = Milliseconds::from_hours(expected_time_hours).0;

        let dist = Haversine::distance_km(src.x.0, src.y.0, dst.x.0, dst.y.0);

        match h.cost(src, dst) {
            Err(e) => {
                println!("{}", e.to_string());
                panic!();
            }
            Ok(time) => {
                assert_eq!(dist, expected_dist);
                assert_eq!(time.0, expected_time);
            }
        }
    }
}
