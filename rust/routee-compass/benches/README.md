# RouteE-Compass Benchmarks

RouteE-Compass Benchmarks using `criterion` and `github-action-benchmark`.

The results from the benchmark workflow are saved in the repository on the `gh-pages` branch. The results are added to the `dev/bench/` folder after each workflow run. The benchmarks are plotted at `nrel.github.io/routee-compass/dev/bench`. See the [rust-bench](../../../.github/workflows/rust-bench.yaml) workflow and [github-action-benchmark](https://github.com/benchmark-action/github-action-benchmark) for more details.


To run the benchmarks manually
```
cd rust/
cargo bench
```

> Note passing many command-line arguments requires disabling bench building in the `Cargo.toml`. See [criterion docs](https://bheisler.github.io/criterion.rs/book/faq.html#cargo-bench-gives-unrecognized-option-errors-for-valid-command-line-options) for more information.