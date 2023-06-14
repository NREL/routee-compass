use crate::model::property::vertex::Vertex;

use crate::model::cost::{cost::Cost, cost_error::CostError};

pub trait CostEstimateFunction: Sync + Send {
    fn cost(&self, src: Vertex, dst: Vertex) -> Result<Cost, CostError>;
}

/// the default cost estimate function uses the haversine formula to give us
/// the lower bound on distance between two vertices. we then "traverse" that
/// distance using the provided travel speed in kilometers per hour.
pub struct Haversine {
    travel_speed_kph: f64,
}



impl CostEstimateFunction for Haversine {
    fn cost(&self, src: Vertex, dst: Vertex) -> Result<Cost, CostError> {
        let d_lat: f64 = (dst.y.0 - src.y.0).to_radians();
        let d_lon: f64 = (dst.x.0 - src.x.0).to_radians();
        let lat1: f64 = (src.y.0).to_radians();
        let lat2: f64 = (dst.y.0).to_radians();

        let a: f64 = ((d_lat / 2.0).sin()) * ((d_lat / 2.0).sin())
            + ((d_lon / 2.0).sin()) * ((d_lon / 2.0).sin()) * (lat1.cos()) * (lat2.cos());
        let c: f64 = 2.0 * ((a.sqrt()).atan2((1.0 - a).sqrt()));

        let distance_kph = c * 6371.0;
        let travel_time = distance_kph / self.travel_speed_kph;
        Ok(Cost(travel_time as i64))
    }
}
