# RouteE-Compass Benchmarks

RouteE-Compass Benchmarks using `criterion` and `github-action-benchmark`.

The results from the benchmark workflow are saved in the repository on the `gh-pages` branch. The results are added to the `dev/bench/` folder after each workflow run. The benchmarks are plotted at `nrel.github.io/routee-compass/dev/bench`. See the [rust-bench](../../../.github/workflows/rust-bench.yaml) workflow and [github-action-benchmark](https://github.com/benchmark-action/github-action-benchmark) for more details.


To run the benchmarks manually
```
cd rust/
cargo criterion
```
> [cargo-criterion](https://github.com/bheisler/cargo-criterion) is used to provide machine-readable output.

## A note about relative paths with `cargo criterion`
 `cargo-criterion` finds relative paths based on the current working directory, while the command `cargo bench` finds the relative paths based on the package location.