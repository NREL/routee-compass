# Installation

## python

### PyPI

You can install this package from PyPI using pip:

```bash
pip install nrel.routee.compass
```

Or, if you want plotting and OSM features:

```bash
pip install 'nrel.routee.compass[all]'
```

### from source

You can also install from source by running the following from the root folder of this repository:

```bash
pip install '.[all]'
```

## rust

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

Building the application from source can be done using `cargo`:

```bash
git clone https://github.com/NREL/routee-compass.git

cd routee-compass/rust

cargo build --release
```

This will build the application into the location `path/to/routee-compass/rust/target/release/routee-compass`

You can optionally alias the application to make it easier to run:

```bash
alias compass-app=path/to/routee-compass/rust/target/release/routee-compass
```
