use crate::app::compass::CompassAppError;
use geo::LineString;
use kdam::Bar;
use routee_compass_core::util::fs::read_utils;
use routee_compass_core::util::geo::geo_io_utils::parse_wkt_linestring;
use std::io::ErrorKind;

pub struct GeomAppConfig {
    pub edge_file: String,
}

pub struct GeomApp {
    geoms: Box<[LineString<f32>]>,
}

impl TryFrom<&GeomAppConfig> for GeomApp {
    type Error = CompassAppError;

    ///
    /// builds a GeomApp instance. this reads and decodes a file with LINESTRINGs enumerated
    /// by their row index, starting from zero, treated as EdgeIds.
    /// the app can then process a file which provides a list of EdgeIds and return the corresponding LINESTRINGs.
    fn try_from(conf: &GeomAppConfig) -> Result<Self, Self::Error> {
        let op = |idx: usize, row: String| {
            let result = parse_wkt_linestring(idx, row)?;
            Ok(result)
        };

        let geoms = read_utils::read_raw_file(
            &conf.edge_file,
            op,
            Some(Bar::builder().desc("link geometries")),
            None,
        )
        .map_err(|e| {
            CompassAppError::BuildFailure(format!(
                "failure reading edge file {}: {}",
                conf.edge_file, e
            ))
        })?;
        eprintln!();
        let app = GeomApp { geoms };
        Ok(app)
    }
}

impl GeomApp {
    /// run the GeomApp. reads each line of a file, which is expected to be a number coorelating to
    /// some EdgeId. looks up the geometry for that EdgeId.
    pub fn run(&self, file: String) -> Result<Box<[LineString<f32>]>, CompassAppError> {
        let op = |idx: usize, row: String| {
            let edge_idx = row
                .parse::<usize>()
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            let result = self.geoms.get(edge_idx).cloned().ok_or_else(|| {
                std::io::Error::new(
                    ErrorKind::InvalidData,
                    format!("EdgeId {idx} is out of bounds, should be in range [0, )"),
                )
            });
            result
        };

        let result: Box<[LineString<f32>]> = read_utils::read_raw_file(
            &file,
            op,
            Some(Bar::builder().desc("link geometries")),
            None,
        )
        .map_err(|e| {
            CompassAppError::BuildFailure(format!("failure reading linestring file {file}: {e}"))
        })?;
        eprintln!();
        Ok(result)
    }
}
