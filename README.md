# routee-compass

RouteE Compass is an energy-aware routing engine for the RouteE ecosystem of software tools with the following key features:

 - Dynamic and extensible search objectives that allow customized blends of distance, time, cost, and energy (via RouteE Powertrain) at query-time
 - Core engine written in Rust for improved runtimes, parallel query execution, and the ability to load nation-sized road networks into memory
 - Rust, HTTP, and Python APIs for integration into different research pipelines and other software

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

Three workflows for running RouteE Compass are detailed below:
  - [Python using OpenStreetMaps networks](#run-with-openstreetmaps)
  - [Python using a custom backend](#python-using-a-custom-backend)
  - [Command line application](#command-line-application)

### Python using OpenStreetMaps networks

The Python CompassApp comes equipped with a tool to download OpenStreetMaps networks and reformat them as RouteE Compass datasets.
See [the example notebook](examples/OpenStreetMaps%20Example.ipynb) for a simple walkthrough.

### Python using a custom backend

This workflow depends on importing your own custom road network datasets.
The workflow for executing queries is the same as the process for using OpenStreetMaps as described above.

##### data preparation requirements
_todo_

##### configuration file

The application expects a config file to tell it where to find the graph data and what traversal model it should use.
Take a look at [the default configuation](./rust/compass-app/src/app/compass/config/config.default.toml) to see an example of what this looks like.

##### query

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

##### python execution

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

### Command line application

Once you've built the application, you can also run it from the command line, passing in your config and your [query](#query):

```bash
path/to/routee-compass/rust/target/release/compass-app --config path/to/config.toml path/to/query.json
```

This will load the graph and then run the query (or queries) from your `query.json` file, outputing results to a file called `results.json` in the current working directory.
