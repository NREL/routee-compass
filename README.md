# routee-compass 
A routing engine that considers energy weights on edges of a graph for particular vehicle types - built for integration with RouteE.

# setup 

For prototyping, this uses some local wheels that need to be installed manually:

```bash
conda env create -f environment.yml
conda activate routee-compass 

pip install -e .

pip install lib/routee=0.3.1/routee-0.3.1-py3-none-any.whl
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
