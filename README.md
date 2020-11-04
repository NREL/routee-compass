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

run `scripts/get_osm_road_network.py [infile] [outfile]`:

```bash
cd scripts
python get_osm_road_network.py denver_metro/denver_metro.shp ../resources/denver_metro_osm_roadnetwork.pickle
```

### tomtom 

to download an parse a tomtom network you'll have to install the following packages:

```bash
conda install geopandas sqlalchemy psycopg2
```

then, you'll also need a polygon shapefile that defines the boundaries of your road network (see `scripts/denver_metro`)

run `scripts/get_tomtom_road_network.py [infile] [outfile]`:

```bash
cd scripts
python get_tomtom_road_network.py denver_metro/denver_metro.shp ../resources/denver_metro_tomtom_roadnetwork.pickle
```

note: you'll need access to the trolley postgres server.
