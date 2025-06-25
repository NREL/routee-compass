//! Benchmark the performance of the downtown denver example
//!
//! ```
//! cd rust/
//! cargo criterion
//! ```
//!
//! If you use the command `cargo bench`, then you will have to change the `config_file` to
//! ```
//! // this path is relative to `routee-compass/rust/routee-compass`
//! config_file: String::from(
//!     "../../python/nrel/routee/compass/resources/downtown_denver_example/osm_default_speed.toml",
//! ),
//! ```

use std::{hint::black_box, io::Write};

use routee_compass::app::cli::cli_args::CliArgs;
use routee_compass::app::cli::run;
use routee_compass::app::compass::CompassAppBuilder;

use criterion::{criterion_group, criterion_main, Criterion};
use tempfile::NamedTempFile;

/// Run the query on the downtown denver example config file
fn downtown_denver_example(query_file: String) {
    let args = CliArgs {
        // this path is relative to `routee-compass/rust`
        config_file: String::from(
            "../python/nrel/routee/compass/resources/downtown_denver_example/osm_default_speed.toml",
        ),
        query_file: query_file,
        chunksize: None,
        newline_delimited: false,
    };
    let builder = CompassAppBuilder::default();
    match run::command_line_runner(&args, Some(builder), None) {
        Ok(_) => {}
        Err(e) => {
            println!("{}", e)
        }
    }
}

/// Benchmark the downtown denver example using criterion
fn bench_example(c: &mut Criterion) {
    let mut group = c.benchmark_group("routee-compass");

    let query = "{
        \"origin_name\": \"NREL\",
        \"destination_name\": \"Comrade Brewing Company\",
        \"destination_y\": 39.62627481432341,
        \"destination_x\": -104.99460207519721,
        \"origin_y\": 39.798311884359094,
        \"origin_x\": -104.86796368632217
    }";

    let mut tmp_file = NamedTempFile::new().unwrap();
    tmp_file.write_all(query.as_bytes()).unwrap();
    let tmp_path = tmp_file.into_temp_path();

    group.bench_with_input("downtown denver example", &tmp_path, |b, input| {
        b.iter(|| {
            black_box(downtown_denver_example(black_box(
                input.to_str().unwrap().to_string(),
            )))
        })
    });
}

criterion_group!(benches, bench_example);
criterion_main!(benches);
