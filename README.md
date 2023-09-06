# routee-compass

RouteE-Compass is a robust, energy-aware, routing engine.

The routing engine has been designed to integrate with the other tools in the RouteE ecosystem and has the following key features:

- Dynamic costing that allows for any vehicle model to be used (represented by a RouteE-Powertrain model)
- Python API for integration into research pipelines and other python software
- Core engine written in Rust for improved performance

## setup

### get rust

The core engine is written in Rust and so we'll need access to a Rust compiler.
There are a couple of ways to do this:

#### rustup

The recommended way to install rust is to [use rustup](https://www.rust-lang.org/tools/install).

#### conda

An alternative way to get rust is to use the anaconda package manager:

```bash
conda create -n routee-compass python=3.10 rust
conda activate routee-compass
```

### build

#### python

You can build the application as a python library by running the following from the root folder of this repository:

```bash
pip install .
```

#### from source

Building the application from source can be done using `cargo`:

```bash
git clone https://github.nrel.gov/MBAP/routee-compass.git

cd routee-compass/rust

cargo build --release
```

This will build the application into the location `path/to/routee-compass/rust/target/release/compass-app`

You can optionally alias the application to make it easier to run:

```bash
alias compass-app=path/to/routee-compass/rust/target/release/compass-app
```

## running

### config

The application expects a config file to tell it where to find the graph data and what traversal model it should use.
Take a look at [the default configuation](./rust/compass-app/src/app/compass/config/config.default.toml) to see an example of what this looks like.

### query

In addition to an application level config, we also need to specify what queries the application should run.
These are represented as json that can contain one or multiple queries. Here's an example:

```json
{
  "origin_name": "NREL",
  "destination_name": "Comrade Brewing Company",
  "origin_x": -105.1710052,
  "origin_y": 39.7402804,
  "destination_x": -104.9009913,
  "destination_y": 39.6757025
}
```

### python library

If you installed the application using `pip`, you can load it and run queries within python:

```python
from nrel.routee.compass import CompassApp

app = CompassApp.from_config_file("path/to/config.toml")

query = {
    "origin_name": "NREL",
    "destination_name": "Comrade Brewing Company",
    "origin_x": -105.1710052,
    "origin_y": 39.7402804,
    "destination_x": -104.9009913,
    "destination_y": 39.6757025
}

# result here is a list of python dictionaries
results = app.run(query)
```

### command line application

Once you've built the application, you can run it from the command line, passing in your config and your query

```bash
path/to/routee-compass/rust/target/release/compass-app --config path/to/config.toml path/to/query.json
```

This will load the graph and then run the query (or queries) from your `query.json` file, outputing results to a file called `results.json` in the current working directory.
