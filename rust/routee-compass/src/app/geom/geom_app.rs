use crate::app::compass::compass_app_error::CompassAppError;
use geo::LineString;
use kdam::Bar;
use kdam::BarExt;
use routee_compass_core::util::fs::{fs_utils, read_utils};
use routee_compass_core::util::geo::geo_io_utils::parse_linestring;
use std::io::ErrorKind;

pub struct GeomAppConfig {
    pub edge_file: String,
}

pub struct GeomApp {
    geoms: Box<[LineString]>,
}

impl TryFrom<&GeomAppConfig> for GeomApp {
    type Error = CompassAppError;

    ///
    /// builds a GeomApp instance. this reads and decodes a file with LINESTRINGs enumerated
    /// by their row index, starting from zero, treated as EdgeIds.
    /// the app can then process a file which provides a list of EdgeIds and return the corresponding LINESTRINGs.
    fn try_from(conf: &GeomAppConfig) -> Result<Self, Self::Error> {
        let count =
            fs_utils::line_count(conf.edge_file.clone(), fs_utils::is_gzip(&conf.edge_file))
                .map_err(CompassAppError::IOError)?;

        let mut pb = Bar::builder()
            .total(count)
            .animation("fillup")
            .desc("geometry file")
            .build()
            .map_err(CompassAppError::UXError)?;

        let cb = Box::new(|| {
            let _ = pb.update(1);
        });

        let op = |idx: usize, row: String| {
            let result = parse_linestring(idx, row)?;
            Ok(result)
        };

        let geoms = read_utils::read_raw_file(&conf.edge_file, op, Some(cb))
            .map_err(CompassAppError::IOError)?;
        println!();
        let app = GeomApp { geoms };
        Ok(app)
    }
}

impl GeomApp {
    /// run the GeomApp. reads each line of a file, which is expected to be a number coorelating to
    /// some EdgeId. looks up the geometry for that EdgeId.
    pub fn run(&self, file: String) -> Result<Box<[LineString]>, CompassAppError> {
        let count = fs_utils::line_count(file.clone(), fs_utils::is_gzip(&file))
            .map_err(CompassAppError::IOError)?;

        let mut pb = Bar::builder()
            .total(count)
            .animation("fillup")
            .desc("edge id list")
            .build()
            .map_err(CompassAppError::UXError)?;

        let cb = Box::new(|| {
            let _ = pb.update(1);
        });

        let op = |idx: usize, row: String| {
            let edge_idx = row
                .parse::<usize>()
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            let result = self.geoms.get(edge_idx).cloned().ok_or({
                std::io::Error::new(
                    ErrorKind::InvalidData,
                    format!("EdgeId {} is out of bounds, should be in range [0, )", idx),
                )
            });
            result
        };

        let result: Box<[LineString]> =
            read_utils::read_raw_file(&file, op, Some(cb)).map_err(CompassAppError::IOError)?;
        Ok(result)
    }
}
