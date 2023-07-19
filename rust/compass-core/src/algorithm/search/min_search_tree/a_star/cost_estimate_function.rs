use geo::Coord;
use uom::si;
use crate::model::units::{Length, Velocity};

use crate::model::cost::{cost::Cost, cost_error::CostError};
use crate::model::property::vertex::Vertex;

pub trait CostEstimateFunction: Sync + Send {
    fn cost(&self, src: Vertex, dst: Vertex) -> Result<Cost, CostError>;
}

/// the default cost estimate function uses the haversine formula to give us
/// the lower bound on distance between two vertices. we then "traverse" that
/// distance using the provided travel speed in kilometers per hour.
pub struct Haversine {
    pub travel_speed: Velocity,
}

impl CostEstimateFunction for Haversine {
    /// uses the haversine distance between two points to estimate a lower bound of
    /// travel time in milliseconds.
    fn cost(&self, src: Vertex, dst: Vertex) -> Result<Cost, CostError> {
        let distance = Haversine::distance(src.coordinate, dst.coordinate);
        let travel_time = distance / self.travel_speed;
        let travel_time_ms = travel_time.get::<si::time::millisecond>();
        Ok(Cost::from_f64(travel_time_ms))
    }
}

impl Haversine {
    pub const APPROX_EARTH_RADIUS_KM: f64 = 6372.8;

    /// haversine distance formula, based on the one published to rosetta code.
    /// https://rosettacode.org/wiki/Haversine_formula#Rust
    /// computes the great circle distance between two points in kilometers.
    pub fn distance(src: Coord, dst: Coord) -> Length {
        let lat1 = src.y.to_radians();
        let lat2 = dst.y.to_radians();
        let d_lat = lat2 - lat1;
        let d_lon = (dst.x - src.x).to_radians();

        let a = (d_lat / 2.0).sin().powi(2) + (d_lon / 2.0).sin().powi(2) * lat1.cos() * lat2.cos();
        let c = 2.0 * a.sqrt().asin();
        let distance_km = Haversine::APPROX_EARTH_RADIUS_KM * c;
        let distance = Length::new::<uom::si::length::kilometer>(distance_km);
        distance
    }
}

#[cfg(test)]
mod tests {
    use geo::coord;
    use uom::si;
    use crate::model::units::{Length, Velocity};

    use crate::model::{
        graph::vertex_id::VertexId, property::vertex::Vertex
    };

    use super::{CostEstimateFunction, Haversine};

    #[test]
    fn test_haversine() {
        // based on test case at the rosetta code website:
        // https://rosettacode.org/wiki/Haversine_formula#Rust

        let h = Haversine {
            travel_speed: Velocity::new::<si::velocity::kilometer_per_hour>(40.0),
        };
        let src = Vertex {
            vertex_id: VertexId(0),
            coordinate: coord! {x: -86.67, y: 36.12},
        };
        let dst = Vertex {
            vertex_id: VertexId(1),
            coordinate: coord! {x: -118.40, y: 33.94},
        };

        let expected_dist = Length::new::<si::length::kilometer>(2887.2599506071106);
        let expected_time = expected_dist / h.travel_speed;

        let dist = Haversine::distance(src.coordinate, dst.coordinate);

        match h.cost(src, dst) {
            Err(e) => {
                println!("{}", e.to_string());
                panic!();
            }
            Ok(time) => {
                assert_eq!(dist, expected_dist);
                assert_eq!(
                    time.into_f64(),
                    expected_time.get::<si::time::millisecond>()
                );
            }
        }
    }
}
