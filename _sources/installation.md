# Installation

## python

(conda-install)=

### conda (recommended)

The recommended way to get started with RouteE Compass is to use [conda](https://docs.conda.io/en/latest/) to install the python package:

```console
conda create -n routee-compass -c conda-forge python=3.11 nrel.routee.compass
```

This creates a new conda environment with python 3.11 and then installs RouteE Compass into it.

The conda distribution includes several optional dependencies and is recommended if you want everything included or don't already have any road network data on your system.

(pip-install)=

### pip

You can also just directly install the package directly with [pip](https://pypi.org/project/pip/) and none of the optional dependnecies:

```console
pip install nrel.routee.compass
```

This is useful if you already have a road network dataset (see [here](notebooks/open_street_maps_example.ipynb)) on your system and you just want to compute routes.

(python-from-source)=

### from source

You can also install the python package from source with a couple of extra steps.

The core engine is written in Rust and so we'll need access to a Rust compiler.
There are a couple of ways to do this:

1. The recommended way to install rust is to [use rustup](https://www.rust-lang.org/tools/install).

1. An alternative way to get rust is to use the anaconda package manager `conda install rust`.

Once you have rust available on your system, you can use [pip](https://pypi.org/project/pip/) to install the package from source while in the repository root directory:

```console
pip install .
```

## rust

In addition to the python package, you can also build the core rust application from source and run the application from the command line.

Assuming you have rust on your system (see the [from source](python-from-source) section above), you can build the application using `cargo`:

```console
git clone https://github.com/NREL/routee-compass.git

cd routee-compass/rust

cargo build --release
```

This will build the command line application into the location `path/to/routee-compass/rust/target/release/routee-compass`

You can optionally alias the application to make it easier to run:

```bash
alias routee-compass=path/to/routee-compass/rust/target/release/routee-compass
```
