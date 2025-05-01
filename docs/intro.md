# RouteE Compass

RouteE Compass is an energy-aware routing engine for the RouteE ecosystem of software tools with the following key features:

- Dynamic and extensible search objectives that allow customized blends of distance, time, cost, and energy (via RouteE Powertrain) at query-time
- Core engine written in Rust for improved runtimes, parallel query execution, and the ability to load nation-sized road networks into memory
- Rust, HTTP, and Python APIs for integration into different research pipelines and other software

For more information about Compass, read about our [motivation](motivation)

## Quickstart

1. Follow the [install instructions](installation) for the python package.
1. Follow this [example](examples/01_open_street_maps_example) to start routing over Open Street Maps data.

## Configuration

1. Read about the [recognized unit names](units) that can be referenced in queries and configuration files