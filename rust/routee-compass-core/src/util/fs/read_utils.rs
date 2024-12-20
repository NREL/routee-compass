use super::fs_utils;
use csv::ReaderBuilder;
use flate2::read::GzDecoder;
use kdam::{Bar, BarBuilder, BarExt};

use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

type CsvCallback<'a, T> = Option<Box<dyn FnMut(&T) + 'a>>;
type RawCallback<'a> = Option<Box<dyn FnMut() + 'a>>;

/// environment variable used to denote if the progress bar should be used.
/// if COMPASS_PROGRESS=false, the bar is deactivated, otherwise it runs.
const COMPASS_PROGRESS: &str = "COMPASS_PROGRESS";

/// reads from a CSV into an iterator of T records.
/// building the iterator may fail with an io::Error.
/// each row hasn't yet been decoded so it is provided in a Result<T, csv::Error>
///
pub fn iterator_from_csv<'a, F, T>(
    filepath: F,
    has_headers: bool,
    mut row_callback: CsvCallback<'a, T>,
) -> Result<Box<dyn Iterator<Item = Result<T, csv::Error>> + 'a>, io::Error>
where
    F: AsRef<Path>,
    T: serde::de::DeserializeOwned + 'a,
{
    let f = File::open(filepath.as_ref())?;
    let r: Box<dyn io::Read> = if fs_utils::is_gzip(filepath) {
        Box::new(BufReader::new(GzDecoder::new(f)))
    } else {
        Box::new(f)
    };
    let reader = ReaderBuilder::new()
        .has_headers(has_headers)
        .trim(csv::Trim::Fields)
        .from_reader(r)
        .into_deserialize::<T>()
        .inspect(move |r| {
            if let Ok(t) = r {
                if let Some(cb) = &mut row_callback {
                    cb(t);
                }
            }
        });

    Ok(Box::new(reader))
}

/// reads a csv file into a vector. not space-optimized since size is not
/// known.
pub fn from_csv<'a, T>(
    filepath: &dyn AsRef<Path>,
    has_headers: bool,
    progress: Option<BarBuilder>,
    callback: CsvCallback<'a, T>,
) -> Result<Box<[T]>, csv::Error>
where
    T: serde::de::DeserializeOwned + 'a,
{
    let bar_opt = build_bar(progress);

    let row_callback: CsvCallback<'a, T> = match (callback, bar_opt) {
        (None, None) => None,
        (None, Some(mut bar)) => Some(Box::new(move |_| {
            let _ = bar.update(1);
        })),
        (Some(cb), None) => Some(cb),
        (Some(mut cb), Some(mut bar)) => Some(Box::new(move |t| {
            cb(t);
            let _ = bar.update(1);
        })),
    };

    let iter: Box<dyn Iterator<Item = Result<T, csv::Error>>> =
        iterator_from_csv(filepath, has_headers, row_callback)?;
    let result = iter
        .into_iter()
        .collect::<Result<Vec<T>, csv::Error>>()?
        .into_boxed_slice();
    Ok(result)
}

/// reads in a raw file and deserializes each line of the file into a type T
/// using the provided operation.
/// inspects the file to determine if it should read as a raw or gzip stream.
/// the row index (starting from zero) is passed to the deserialization op
/// as in most cases, the row number is an id.
pub fn read_raw_file<F, T>(
    filepath: F,
    op: impl Fn(usize, String) -> Result<T, io::Error>,
    progress: Option<BarBuilder>,
    callback: Option<Box<dyn FnMut()>>,
) -> Result<Box<[T]>, io::Error>
where
    F: AsRef<Path>,
{
    let bar_opt = build_bar(progress);

    let row_callback: RawCallback = match (callback, bar_opt) {
        (None, None) => None,
        (None, Some(mut bar)) => Some(Box::new(move || {
            let _ = bar.update(1);
        })),
        (Some(cb), None) => Some(cb),
        (Some(mut cb), Some(mut bar)) => Some(Box::new(move || {
            cb();
            let _ = bar.update(1);
        })),
    };

    if fs_utils::is_gzip(filepath.as_ref()) {
        Ok(read_gzip(filepath, op, row_callback)?)
    } else {
        Ok(read_regular(filepath, op, row_callback)?)
    }
}

fn read_regular<'a, F, T>(
    filepath: F,
    op: impl Fn(usize, String) -> Result<T, io::Error>,
    mut row_callback: Option<Box<dyn FnMut() + 'a>>,
) -> Result<Box<[T]>, io::Error>
where
    F: AsRef<Path>,
{
    let file = File::open(filepath)?;
    let reader = BufReader::new(file);
    let result: Result<Box<[T]>, std::io::Error> = reader
        .lines()
        .enumerate()
        .map(|(idx, row)| {
            let parsed = row?;
            let deserialized = op(idx, parsed)?;
            if let Some(cb) = &mut row_callback {
                cb();
            }
            Ok(deserialized)
        })
        .collect();
    result
}

fn read_gzip<'a, F, T>(
    filepath: F,
    op: impl Fn(usize, String) -> Result<T, io::Error>,
    mut row_callback: Option<Box<dyn FnMut() + 'a>>,
) -> Result<Box<[T]>, io::Error>
where
    F: AsRef<Path>,
{
    let file = File::open(filepath)?;
    let mut result = vec![];
    let reader = BufReader::new(GzDecoder::new(file));
    for (idx, row) in reader.lines().enumerate() {
        let parsed = row?;
        let deserialized = op(idx, parsed)?;
        if let Some(cb) = &mut row_callback {
            cb();
        }
        result.push(deserialized);
    }
    Ok(result.into_boxed_slice())
}

/// helper function for building a progress bar.
/// a progress bar is created only if:
///   - the `progress` argument is not None
///   - the logging system is set to DEBUG or INFO
///   - the COMPASS_PROGRESS environment variable is not set to "false"
///
/// # Arguments
///
/// * `progress` - progress bar configuration
///
/// # Returns
///
/// Some progress bar if it should be built, else None
fn build_bar(progress: Option<BarBuilder>) -> Option<Bar> {
    let progress_disabled = std::env::var(COMPASS_PROGRESS)
        .ok()
        .map(|v| v.to_lowercase() == "false")
        .unwrap_or_default();
    let log_info_enabled = log::log_enabled!(log::Level::Info);
    if !progress_disabled && log_info_enabled {
        progress.and_then(|b| b.build().ok())
    } else {
        None
    }
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
        let result = read_raw_file(&filepath, op, None, None).unwrap();
        let expected = vec![
            String::from("RouteE yay"),
            String::from("FASTSim yay"),
            String::from("HIVE yay"),
            String::from("ADOPT yay"),
        ]
        .into_boxed_slice();
        assert_eq!(
            result, expected,
            "result should include each row from the source file"
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
        let result = read_raw_file(&filepath, op, None, None).unwrap();
        let expected = vec![
            String::from("RouteE yay"),
            String::from("FASTSim yay"),
            String::from("HIVE yay"),
            String::from("ADOPT yay"),
        ]
        .into_boxed_slice();
        assert_eq!(
            result, expected,
            "result should include each row from the source file"
        );
    }
}
