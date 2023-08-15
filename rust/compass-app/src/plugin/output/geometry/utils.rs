use geo::{LineString, Point};
use wkt::TryFromWkt;

/// Concatenate a vector of linestrings into a single linestring
///
/// # Arguments
///
/// * `linestrings` - a vector of linestrings
///
/// # Returns
///
/// * a single linestring containing all the points from the input linestrings
///
/// # Example
///
/// ```rust
/// use geo::{LineString, Point};
/// use compass_app::plugin::output::geometry::utils::concat_linestrings;
///
/// let line1 = LineString::from(vec![
///     Point::from((0.0, 0.0)),
///     Point::from((1.0, 1.0)),
/// ]);
/// let line2 = LineString::from(vec![
///     Point::from((3.0, 3.0)),
///     Point::from((4.0, 4.0)),
/// ]);
///
/// let result = concat_linestrings(vec![&line1, &line2]);
/// ```
pub fn concat_linestrings(linestrings: Vec<&LineString>) -> LineString {
    let all_points = linestrings
        .iter()
        .flat_map(|ls| ls.points())
        .collect::<Vec<Point>>();
    LineString::from_iter(all_points)
}

/// Parse a linestring from a string; Used for reading geometry lookup tables
///
/// # Arguments
///
/// * `idx` - the index of the row in the lookup table file
/// * `row` - the row from the file that has the wkt geometry
///
/// # Returns
///
/// * a linestring
pub fn parse_linestring(_idx: usize, row: String) -> Result<LineString, std::io::Error> {
    let geom: LineString = LineString::try_from_wkt_str(row.as_str()).map_err(|e| {
        let msg = format!(
            "failure decoding LineString from lookup table. source: {}; error: {}",
            row,
            e.to_string()
        );
        std::io::Error::new(std::io::ErrorKind::InvalidData, msg)
    })?;
    Ok(geom)
}
