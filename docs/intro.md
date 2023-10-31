# RouteE Compass

RouteE Compass is an energy-aware routing engine for the RouteE ecosystem of software tools with the following key features:

- Dynamic and extensible search objectives that allow customized blends of distance, time, cost, and energy (via RouteE Powertrain) at query-time
- Core engine written in Rust for improved runtimes, parallel query execution, and the ability to load nation-sized road networks into memory
- Rust, HTTP, and Python APIs for integration into different research pipelines and other software

## Quickstart

The fastest way to get started with RouteE Compass is to use the python package manager `pip`:

```bash
pip install 'nrel.routee.compass[all]'
```

Then, follow this [example](notebooks/open_street_maps_example.ipynb) to start routing over Open Street Maps data.
