use std::{
    fs::File,
    io::{self, BufRead, BufReader},
};

use flate2::read::GzDecoder;

use super::fs_utils;

/// reads in a raw file and deserializes each line of the file into a type T
/// using the provided operation.
/// inspects the file to determine if it should read as a raw or gzip stream.
/// the row index (starting from zero) is passed to the deserialization op
/// as in most cases, the row number is an id.
pub fn read_raw_file<T>(
    file: &File,
    op: impl Fn(usize, String) -> Result<T, io::Error>,
) -> Result<Vec<T>, io::Error> {
    if fs_utils::is_gzip(file) {
        return Ok(read_gzip(file, op)?);
    } else {
        return Ok(read_regular(file, op)?);
    }
}

fn read_regular<'a, T>(
    file: &File,
    op: impl Fn(usize, String) -> Result<T, io::Error>,
) -> Result<Vec<T>, io::Error> {
    let mut result: Vec<T> = vec![];
    let reader = BufReader::new(file);
    for (idx, row) in reader.lines().enumerate() {
        let parsed = row?;
        let deserialized = op(idx, parsed)?;
        result.push(deserialized);
    }
    return Ok(result);
}

fn read_gzip<'a, T>(
    file: &File,
    op: impl Fn(usize, String) -> Result<T, io::Error>,
) -> Result<Vec<T>, io::Error> {
    let mut result: Vec<T> = vec![];
    let reader = BufReader::new(GzDecoder::new(file));
    for (idx, row) in reader.lines().enumerate() {
        let parsed = row?;
        let deserialized = op(idx, parsed)?;
        result.push(deserialized);
    }
    return Ok(result);
}
