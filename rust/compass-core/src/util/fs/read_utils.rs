use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

use flate2::read::GzDecoder;

use super::fs_utils;

/// reads in a raw file and deserializes each line of the file into a type T
/// using the provided operation.
/// inspects the file to determine if it should read as a raw or gzip stream.
/// the row index (starting from zero) is passed to the deserialization op
/// as in most cases, the row number is an id.
pub fn read_raw_file<F, T>(
    filepath: &F,
    op: impl Fn(usize, String) -> Result<T, io::Error>,
) -> Result<Vec<T>, io::Error>
where
    F: AsRef<Path>,
{
    if fs_utils::is_gzip(filepath) {
        return Ok(read_gzip(filepath, op)?);
    } else {
        return Ok(read_regular(filepath, op)?);
    }
}

fn read_regular<'a, F, T>(
    filepath: &F,
    op: impl Fn(usize, String) -> Result<T, io::Error>,
) -> Result<Vec<T>, io::Error>
where
    F: AsRef<Path>,
{
    let file = File::open(filepath)?;
    let reader = BufReader::new(file);
    let result: Result<Vec<T>, std::io::Error> = reader
        .lines()
        .enumerate()
        .map(|(idx, row)| {
            let parsed = row?;
            let deserialized = op(idx, parsed)?;
            Ok(deserialized)
        })
        .collect();
    return result;
}

fn read_gzip<'a, F, T>(
    filepath: &F,
    op: impl Fn(usize, String) -> Result<T, io::Error>,
) -> Result<Vec<T>, io::Error>
where
    F: AsRef<Path>,
{
    let file = File::open(filepath)?;
    let mut result: Vec<T> = vec![];
    let reader = BufReader::new(GzDecoder::new(file));
    for (idx, row) in reader.lines().enumerate() {
        let parsed = row?;
        let deserialized = op(idx, parsed)?;
        result.push(deserialized);
    }
    return Ok(result);
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::read_raw_file;

    #[test]
    fn test_read_raw_file() {
        let filepath = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("util")
            .join("fs")
            .join("test")
            .join("test.txt");
        println!("loading file {:?}", filepath);
        let bonus_word = " yay";
        let op = |_idx: usize, row: String| Ok(row + bonus_word);
        let result = read_raw_file(&filepath, op).unwrap();
        assert_eq!(
            result,
            vec!["RouteE yay", "FASTSim yay", "HIVE yay", "ADOPT yay"],
            "result should include each row from the source file along with the bonus word"
        );
    }

    #[test]
    fn test_read_raw_file_gzip() {
        let filepath = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("util")
            .join("fs")
            .join("test")
            .join("test.txt.gz");
        println!("loading file {:?}", filepath);
        let bonus_word = " yay";
        let op = |_idx: usize, row: String| Ok(row + bonus_word);
        let result = read_raw_file(&filepath, op).unwrap();
        assert_eq!(
            result,
            vec!["RouteE yay", "FASTSim yay", "HIVE yay", "ADOPT yay"],
            "result should include each row from the source file along with the bonus word"
        );
    }
}
