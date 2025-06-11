use geo::Coord;
use uom::si::f64::Length;
// pub const APPROX_EARTH_RADIUS_KM: f64 = 6372.8;
pub const APPROX_EARTH_RADIUS_M: f32 = 6_371_000.0;

/// get the distance between two coordinates and return the value
/// coordinates are assumed to be in the WGS84 Coordinate System.
pub fn coord_distance(src: &Coord<f32>, dst: &Coord<f32>) -> Result<Length, String> {
    let distance_meters: Length = haversine_distance(src.x, src.y, dst.x, dst.y)?;
    Ok(distance_meters)
}

/// haversine distance formula, based on the one published to rosetta code.
/// https://rosettacode.org/wiki/Haversine_formula#Rust
/// computes the great circle distance between two points in meters.
/// assumes input data is in WGS84 projection (aka EPSG:4326 CRS)
pub fn haversine_distance(
    src_x: f32,
    src_y: f32,
    dst_x: f32,
    dst_y: f32,
) -> Result<Length, String> {
    if !(-180.0..=180.0).contains(&src_x) {
        return Err(format!("src x value not in range [-180, 180]: {}", src_x));
    }
    if !(-180.0..=180.0).contains(&dst_x) {
        return Err(format!("dst x value not in range [-180, 180]: {}", dst_x));
    }
    if !(-90.0..=90.0).contains(&src_y) {
        return Err(format!("src y value not in range [-90, 90]: {}", src_y));
    }
    if !(-90.0..=90.0).contains(&dst_y) {
        return Err(format!("dst y value not in range [-90, 90]: {}", dst_y));
    }

    let lat1 = src_y.to_radians();
    let lat2 = dst_y.to_radians();
    let d_lat = lat2 - lat1;
    let d_lon = (dst_x - src_x).to_radians();

    let a = (d_lat / 2.0).sin().powi(2) + (d_lon / 2.0).sin().powi(2) * lat1.cos() * lat2.cos();
    let c = 2.0 * a.sqrt().asin();
    let distance_meters_f64: f64 = (APPROX_EARTH_RADIUS_M * c).into();
    let distance = Length::new::<uom::si::length::meter>(distance_meters_f64);
    Ok(distance)
}
