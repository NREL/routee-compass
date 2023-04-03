# routee-compass

A routing engine that considers energy weights on edges of a graph for particular vehicle types - built for integration with RouteE.

## setup

### from pip

```bash
pip install nrel.routee.compass --extra-index-url=https://github.nrel.gov/pages/MBAP/mbap-pypi/
```

### from source

```bash
git clone https://github.nrel.gov/MBAP/routee-compass.git
cd routee-compass

pip install .
```

### get a road network

We support the tomtom current road network.

```bash
cd scripts
python download_road_map.py <path/to/polygon.geojson> <my-road-network.json> 
```

note: you'll need access to the trolley postgres server.

## start routing

Once you have a road network file downloaded you can start computing least energy routes.

Here's a sample workflow for loading the road network and finding the least energy path:

```python
from nrel.routee.compass.compass_map import CompassMap
from nrel.routee.compass.rotuee_model_collection import RouteeModelCollection

from mappymatch.constructs.coordinate import Coordinate

road_network = CompassMap.from_file("path/to/my/tomtom_road_network.json")

routee_models = RouteeModelCollection()

road_network.compute_energy(routee_models)

origin = Coordinate.from_lat_lon(lat=39.00, lon=-104.00)
destination = Coordinate.from_lat_lon(lat=39.10, lon=-104.10)

shortest_energy_route = road_network.route(origin, destination, routee_key="Electric") 
```

The road network will compute energy over the whole graph so it could take some time if the graph is large.

Note that routee-compass comes with two default routee-powertrain models "Gasoline" and "Electric".

If you want to use your own routee models you can do so like this:

```python
from nrel.routee.compass.compass_map import CompassMap
from nrel.routee.compass.rotuee_model_collection import RouteeModelCollection

from mappymatch.constructs.coordinate import Coordinate

my_routee_models = {
    "Tesla": "path/to/tesla_model.json",
    "Ferrari": "path/to/ferrari_model.json",
} 
routee_models = RouteeModelCollection(my_routee_models)

road_network = CompassMap.from_file("path/to/my/tomtom_road_network.json")

road_network.compute_energy(routee_models)

origin = Coordinate(lat=39.00, lon=-104.00)
destination = Coordinate(lat=39.10, lon=-104.10)

tesla_shortest_energy_route = road_network.route(origin, destination, routee_key="Tesla")
ferrari_shortest_energy_route = road_network.route(origin, destination, routee_key="Ferrari")
```
