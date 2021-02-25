# routee-compass 
A routing engine that considers energy weights on edges of a graph for particular vehicle types - built for integration with RouteE.

# setup 

## prepare a python environment

```bash
conda create -n routee-compass python=3.8 
conda activate routee-compass 

git clone https://github.nrel.gov/MBAP/routee-compass.git
cd routee-compass

pip install -e .
```

## get a road network

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

# start routing 

Once you have a road network file downloaded you can start computing least energy routes.

Here's a sample workflow for loading the road network and finding the least energy path:

```python
from compass.road_network.tomtom_networkx import TomTomNetworkX
from compass.utils.geo_utils import Coordinate

road_network = TomTomNetworkX("path/to/my/tomtom_road_network.pickle")

origin = Coordinate(lat=39.00, lon=-104.00)
destination = Coordinate(lat=39.10, lon=-104.10)

shortest_energy_route, route_metadata = road_network.shortest_path(origin, destination, routee_key="Electric") 
```
The road network will compute energy over the whole graph when it's loaded so it could take some time if the graph is large.

Note that routee-compass comes with two default routee-powertrain models "Gasoline" and "Electric".

If you want to use your own routee models you can do so like this:

```python
from compass.road_network.tomtom_networkx import TomTomNetworkX
from compass.utils.geo_utils import Coordinate
from compass.utils.routee_utils import RouteeModelCollection

my_routee_models = {
    "Tesla": "path/to/tesla_model.json",
    "Ferrari": "path/to/ferrari_model.json",
} 

road_network = TomTomNetworkX("path/to/my/tomtom_road_network.pickle", RouteeModelCollection(my_routee_models))

origin = Coordinate(lat=39.00, lon=-104.00)
destination = Coordinate(lat=39.10, lon=-104.10)

tesla_shortest_energy_route, tesla_route_metadata = road_network.shortest_path(origin, destination, routee_key="Tesla")
ferrari_shortest_energy_route, ferrari_route_metadata = road_network.shortest_path(origin, destination, routee_key="Ferrari")
```

