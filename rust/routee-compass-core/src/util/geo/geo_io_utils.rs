use geo::{Coord, CoordsIter, LineString, Point};
use geo_traits::to_geo::ToGeoGeometry;
use itertools::Itertools;
use wkb;
use wkt::{ToWkt, TryFromWkt};

/// downsamples an f64 geometry to f32.
/// we currently use f32 to reduce the memory footprint of some map geometry data.
pub fn downsample_geometry(geometry_f64: geo::Geometry<f64>) -> Result<geo::Geometry<f32>, String> {
    match geometry_f64 {
        geo::Geometry::Polygon(p) => {
            let ext = p
                .exterior_coords_iter()
                .map(|c| Coord::<f32> {
                    x: c.x as f32,
                    y: c.y as f32,
                })
                .collect_vec();
            let exterior = geo::LineString::new(ext);
            let interiors = p
                .interiors()
                .iter()
                .map(|int| {
                    let int = int
                        .coords()
                        .map(|c| Coord::<f32> {
                            x: c.x as f32,
                            y: c.y as f32,
                        })
                        .collect_vec();
                    geo::LineString::from(int)
                })
                .collect_vec();
            Ok(geo::Geometry::Polygon(geo::Polygon::new(
                exterior, interiors,
            )))
        }
        geo::Geometry::MultiPolygon(mp) => {
            let geoms_f32 = mp
                .into_iter()
                .map(|p| downsample_geometry(geo::Geometry::Polygon(p)))
                .collect::<Result<Vec<_>, _>>()?;
            let polys = geoms_f32
                .into_iter()
                .enumerate()
                .map(|(idx, g)| match g {
                    geo::Geometry::Polygon(polygon) => Ok(polygon),
                    _ => Err(format!(
                        "invalid multipolygon contains non-POLYGON geometry at index {idx}"
                    )),
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(geo::Geometry::MultiPolygon(geo::MultiPolygon::new(polys)))
        }
        _ => Err(String::from("not (yet) implemented for this geometry type")),
    }
}

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
        let msg =
            format!("failure decoding LineString from lookup table. source: {row}; error: {e}");
        std::io::Error::new(std::io::ErrorKind::InvalidData, msg)
    })?;
    Ok(geom)
}

pub fn parse_wkb_linestring(_idx: usize, row: String) -> Result<LineString<f32>, std::io::Error> {
    let geom = wkb::reader::read_wkb(row.as_bytes())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

    match geom.to_geometry() {
        geo::Geometry::LineString(l) => {
            // somewhat hackish solution since we cannot choose f32 when parsing wkbs and
            // geo::Convert does not support f64 -> f32, for good reason of course
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
        g => {
            let msg = format!(
                "decoded WKB expected to be linestring, found: {}",
                g.to_wkt()
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
