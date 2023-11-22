use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

use flate2::read::GzDecoder;

/// The output is wrapped in a Result to allow matching on errors
/// Returns an Iterator to the Reader of the lines of the file.
/// based on https://doc.rust-lang.org/rust-by-example/std_misc/file/read_lines.html
///
pub fn line_count<P>(filename: P, is_gzip: bool) -> std::io::Result<usize>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;

    if is_gzip {
        let reader = BufReader::new(GzDecoder::new(file));
        Ok(reader.lines().count())
    } else {
        let reader = BufReader::new(file);
        let count = reader.lines().count();
        Ok(count)
    }
}

/// attempts to read a gzip header from the file. if it is found,
/// then returns true. some inefficiency here due to throwing out the
/// stream object that could have been used later, but in typical Compass
/// settings, this isn't a real bottleneck.
pub fn is_gzip<P>(filepath: P) -> bool
where
    P: AsRef<Path>,
{
    let file_result = File::open(filepath);
    match file_result {
        Err(_) => false,
        Ok(file) => {
            let gz = GzDecoder::new(io::BufReader::new(file));
            gz.header().is_some()
        }
    }
}
