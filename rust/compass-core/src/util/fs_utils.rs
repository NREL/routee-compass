use std::{
    fs::File,
    io::{BufRead, BufReader},
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
        return Ok(reader.lines().count());
    } else {
        let reader = BufReader::new(file);
        return Ok(reader.lines().count());
    }
}
