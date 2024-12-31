use geo::{Coord, LineString, Point};
use itertools::Itertools;
use wkb;
use wkt::{ToWkt, TryFromWkt};

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
/// use routee_compass_core::util::geo::geo_io_utils::concat_linestrings;
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
pub fn concat_linestrings(linestrings: Vec<&LineString<f32>>) -> LineString<f32> {
    let all_points = linestrings
        .iter()
        .flat_map(|ls| ls.points())
        .collect::<Vec<Point<f32>>>();
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
pub fn parse_wkt_linestring(_idx: usize, row: String) -> Result<LineString<f32>, std::io::Error> {
    let geom: LineString<f32> = LineString::try_from_wkt_str(row.as_str()).map_err(|e| {
        let msg = format!(
            "failure decoding LineString from lookup table. source: {}; error: {}",
            row, e
        );
        std::io::Error::new(std::io::ErrorKind::InvalidData, msg)
    })?;
    Ok(geom)
}

pub fn parse_wkb_linestring(_idx: usize, row: String) -> Result<LineString<f32>, std::io::Error> {
    let mut c = row.as_bytes();
    let geom = wkb::wkb_to_geom(&mut c).map_err(|e| {
        let msg = format!("failure decoding WKB string: {:?}", e);
        std::io::Error::new(std::io::ErrorKind::InvalidData, msg)
    })?;
    match geom {
        geo::Geometry::LineString(l) => {
            // somewhat hackish solution since we cannot choose f32 when parsing wkbs
            let coords32 =
                l.0.into_iter()
                    .map(|c| Coord {
                        x: c.x as f32,
                        y: c.y as f32,
                    })
                    .collect_vec();
            let l32 = LineString::new(coords32);
            Ok(l32)
        }
        _ => {
            let msg = format!(
                "decoded WKB expected to be linestring, found: {}",
                geom.to_wkt()
            );
            Err(std::io::Error::new(std::io::ErrorKind::InvalidData, msg))
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_concat_linstrings() {
        let line1 = LineString::from(vec![
            Point::from((0.0, 0.0)),
            Point::from((1.0, 1.0)),
            Point::from((2.0, 2.0)),
        ]);
        let line2 = LineString::from(vec![
            Point::from((3.0, 3.0)),
            Point::from((4.0, 4.0)),
            Point::from((5.0, 5.0)),
        ]);
        let line3 = LineString::from(vec![
            Point::from((6.0, 6.0)),
            Point::from((7.0, 7.0)),
            Point::from((8.0, 8.0)),
        ]);
        let result = concat_linestrings(vec![&line1, &line2, &line3]);
        assert_eq!(result.points().len(), 9);
        let points = result.into_points();
        assert_eq!(points[0], Point::from((0.0, 0.0)));
        assert_eq!(points[8], Point::from((8.0, 8.0)));
    }
}
