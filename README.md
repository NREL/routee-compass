# routee-compass

RouteE Compass is an energy-aware routing engine for the RouteE ecosystem of software tools with the following key features:

- Dynamic and extensible search objectives that allow customized blends of distance, time, cost, and energy (via RouteE Powertrain) at query-time
- Core engine written in Rust for improved runtimes, parallel query execution, and the ability to load nation-sized road networks into memory
- Rust, HTTP, and Python APIs for integration into different research pipelines and other software

## Quickstart

To install RouteE Compass, we must first install Rust:

```bash
conda install -c conda-forge rust
```

The fastest way to get started with RouteE Compass is to use the python package manager `pip`:

```bash
pip install nrel.routee.compass[osm]
```

Then, follow this [example](docs/notebooks/open_street_maps_example.ipynb) to start routing over Open Street Maps data.

TODO: Once we go live point update this:
See the [documentation](https://nrel.github.io/routee-compass/) for more information.

##### configuration file

The application expects a config file to tell it where to find the graph data and what traversal model it should use.
Take a look at [the default configuation](./rust/compass-app/src/app/compass/config/config.default.toml) to see an example of what this looks like.
