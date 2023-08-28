use crate::model::units::Length;
use geo::Coord;
use uom::si;
pub const APPROX_EARTH_RADIUS_KM: f64 = 6372.8;

/// helper function to invoke distance between geo::Coords
pub fn coord_distance_km(src: Coord, dst: Coord) -> Result<Length, String> {
    distance_km(src.x, src.y, dst.x, dst.y)
}

/// haversine distance formula, based on the one published to rosetta code.
/// https://rosettacode.org/wiki/Haversine_formula#Rust
/// computes the great circle distance between two points in kilometers.
/// assumes input data is in WGS84 projection (aka EPSG:4326 CRS)
pub fn distance_km(src_x: f64, src_y: f64, dst_x: f64, dst_y: f64) -> Result<Length, String> {
    if src_x < -180.0 || 180.0 < src_x {
        return Err(format!("src x value not in range [-180, 180]: {}", src_x));
    }
    if dst_x < -180.0 || 180.0 < dst_x {
        return Err(format!("dst x value not in range [-180, 180]: {}", dst_x));
    }
    if src_y < -90.0 || 90.0 < src_y {
        return Err(format!("src y value not in range [-90, 90]: {}", src_y));
    }
    if dst_y < -90.0 || 90.0 < dst_y {
        return Err(format!("dst y value not in range [-90, 90]: {}", dst_y));
    }

    let lat1 = src_y.to_radians();
    let lat2 = dst_y.to_radians();
    let d_lat = lat2 - lat1;
    let d_lon = (dst_x - src_x).to_radians();

    let a = (d_lat / 2.0).sin().powi(2) + (d_lon / 2.0).sin().powi(2) * lat1.cos() * lat2.cos();
    let c = 2.0 * a.sqrt().asin();
    let distance_km = APPROX_EARTH_RADIUS_KM * c;
    let distance = Length::new::<si::length::kilometer>(distance_km);
    return Ok(distance);
}
