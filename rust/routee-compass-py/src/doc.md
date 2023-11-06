# Compass App Py(thon) Crate

This crate provides Python bindings to CompassApp.
For installing RouteE Compass from Python, see the [documentation](../../docs/intro.md) which installs the root-level [python module](../../pyproject.toml).

This crate uses [pyo3](https://github.com/PyO3/pyo3) and [maturin](https://github.com/PyO3/maturin) to create a [thin wrapper] around CompassApp.
Instances of CompassApp loaded in Python still benefit from the underlying rust runtime, for fast parallel execution of lists of queries.

Examples of running queries from python can be found [in the documentation](../../docs/running.md).

[thin wrapper]: crate::app_wrapper::CompassAppWrapper