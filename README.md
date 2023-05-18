# routee-compass

A routing engine that considers energy weights on edges of a graph for particular vehicle types - built for integration with RouteE.

## setup

### conda environment

This codebase uses rust and sometimes it can hard to install (on a remote machine for example). 
You can use conda to install rust into your python virtual evironment like so:

```bash
conda create -n routee-compass python=3.10 rust
conda activate routee-compass
```

### from source

```bash
git clone https://github.nrel.gov/MBAP/routee-compass.git
cd routee-compass

pip install .
```

## developer setup

### from source

```bash
git clone https://github.nrel.gov/MBAP/routee-compass.git
cd routee-compass

pip install -e ".[dev]"
```

### rust extension

Whenever you update any of the rust code, you can re-build the python extension module with maturin 

```bash
pip install maturin
maturin develop --release
```

### get a road network

Right now we don't have a streamlined processes for pulling down a road network and so check out the
`scripts/download_us_network.py` for insight into building the road network.

## start routing

TODO: Fill this back in when we have a streamlined way for building up the road network
