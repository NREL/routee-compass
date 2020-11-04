# routee-compass 
A routing engine that considers energy weights on edges of a graph for particular vehicle types - built for integration with RouteE.

# setup 

```bash
conda create -n routee-compass python=3.8 
conda activate routee-compass 

pip install -e .
```

# road networks

We currently support osm and tomtom networks. 

### osm

to download an parse an osm network you'll have to install the following packages:

```bash
conda install osmnx geopandas
```

then, you'll also need a polygon shapefile that defines the boundaries of your road network (see `scripts/denver_metro`)

```bash
get-osm-network <path/to/my/shapefile.shp> <path/to/outfile/road_network.pickle> 
```

### tomtom 

you'll need a polygon shapefile that defines the boundaries of your road network (see `scripts/denver_metro`)

```bash
get-tomtom-network <path/to/my/shapefile.shp> <path/to/outfile/road_network.pickle> 
```

note: you'll need access to the trolley postgres server.
