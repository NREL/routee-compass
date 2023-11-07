# RouteE Compass

RouteE Compass is an energy-aware routing engine for the RouteE ecosystem of software tools with the following key features:

- Dynamic and extensible search objectives that allow customized blends of distance, time, cost, and energy (via RouteE Powertrain) at query-time
- Core engine written in Rust for improved runtimes, parallel query execution, and the ability to load nation-sized road networks into memory
- Rust, HTTP, and Python APIs for integration into different research pipelines and other software

## Quickstart

1. Follow the [install instructions](conda-install) for the python package.
1. Follow this [example](notebooks/open_street_maps_example.ipynb) to start routing over Open Street Maps data.
