# App Config

Each `CompassApp` instance is defined by a configuration toml file.
The configuration file specifies things like "Which traversal model should I use, and what are its parameters?" and "Which data sources should I use?".

If you follow the [open street maps example](notebooks/open_street_maps_example.ipynb), the code will produce a few configuration files in the `golden_co/` folder. Let's take a look at the `osm_default_energy.toml` file.
We added some annotations to describe the different sections:

```toml
# how many threads should a CompassApp use to process queries?
parallelism = 2

# should we begin the search at either: "vertex" or "edge"
search_orientation = "vertex"

# the parameters for the underlying road network graph
[graph]
# a file containing all the graph edges and their adjacencies
edge_list_input_file = "edges-compass.csv.gz"
# a file containing all the graph verticies
vertex_list_input_file = "vertices-compass.csv.gz"
# if verbose is true, you'll see more information when loading the graph
verbose = true

# which traversal model to use and its parameters
[traversal]
type = "energy_model"
# the units of the speed table
time_model_speed_unit = "kilometers_per_hour"
# the file that has grades for each edge in the graph
grade_table_input_file = "edges-grade-enumerated.txt.gz"
# the units of the grade table
grade_table_grade_unit = "decimal"

# the internal units of the energy model
time_unit = "minutes"
distance_unit = "miles"

# Here we specify the time model to use for the energy model
[traversal.time_model]
type = "speed_table"
# the file that has speeds for each edge in the graph
speed_table_input_file = "edges-posted-speed-enumerated.txt.gz"
# the units of the speed table
speed_unit = "kilometers_per_hour"

# the internal units of the speed table
distance_unit = "miles"
time_unit = "minutes"

# here, we specify which vehicles to make available at query time
# if you wanted to add more models, you would make a new [[traversal.vehicles]] section.
[[traversal.vehicles]]
# the name of the model that can be passed in from a query as "model_name"
name = "2012_Ford_Focus"
# the type of the vehicle, currently either:
# - "ice" i.e. Internal Combustion Engine (ICE)
# - "bev" i.e. Battery Electric Vehicle (BEV)
# - "phev" i.e. Plug-in Hybrid Electric Vehicle (PHEV)
type = "ice"
# the file for the routee-powertrain model
model_input_file = "models/2012_Ford_Focus.bin"
# the units of what the routee-powertrain model expects speed to be in
speed_unit = "miles_per_hour"
# the units of what the routee-powertrain model expects grade to be in
grade_unit = "decimal"
# the units of what the routee-powertrain model outputs for energy
energy_rate_unit = "gallons_gasoline_per_mile"
# the "best case" energy rate for this particular vehicle (something like highway mpg) that's used in the a-star algorithm
ideal_energy_rate = 0.02857143
# A real world adjustment factor for things like temperature and auxillary loads
real_world_energy_adjustment = 1.166

# what underlying machine learn framework to use [smartcore | interpolate | onnx]
# in this case we use a model that interpolates the underlying model type over a regular grid
[traversal.vehicles.model_type.interpolate]
underlying_model_type = "smartcore"
speed_lower_bound = 0
speed_upper_bound = 100
speed_bins = 101
grade_lower_bound = -0.2
grade_upper_bound = 0.2
grade_bins = 41

## The cost section defines how we translate the search state into a cost that is minimized by the algorithm

# The vehicle rates get applied to each component of the cost

# based on 65.5 cents per mile 2023 IRS mileage rate, $/mile
[cost.vehicle_rates.distance]
type = "factor"
factor = 0.655

# based on $20/hr approximation of 2023 median hourly wages, $/second
[cost.vehicle_rates.time]
type = "factor"
factor = 0.333336

# based on AAA regular unleaded gas prices sampled 12/21/2023
[cost.vehicle_rates.energy_liquid]
type = "factor"
factor = 3.120

# based on $0.50/kWh approximation of DCFC charge rates, $/kWhtype = "factor"
[cost.vehicle_rates.energy_electric]
type = "factor"
factor = 0.50

# Each cost component get multiplied by the corresponding vehicle weight.
# So, you could make time more important than distance by increasing the time weight.
[cost.weights]
distance = 1
time = 1
energy_liquid = 1
energy_electric = 1

## Access costs

# A turn delay model that assigns a time cost to each type of turn
[access]
type = "turn_delay"
edge_heading_input_file = "edges-headings-enumerated.csv.gz"
[access.turn_delay_model]
type = "tabular_discrete"
time_unit = "seconds"
[access.turn_delay_model.table]
no_turn = 0.0
slight_right = 0.5
right = 1.0
sharp_right = 1.5
slight_left = 1.0
left = 2.5
sharp_left = 3.5
u_turn = 9.5

# which plugins should be activated?
[plugin]
input_plugins = [
    # The vertex RTree plugin uses an RTree to match coordiantes to graph verticies.
    { type = "vertex_rtree", distance_tolerance = 0.2, distance_unit = "kilometers", vertices_input_file = "vertices-compass.csv.gz" },
    # The grid search allows you to specify a "grid_search" key in the query and it will generate multiple queries from those parameters.
    { type = "grid_search" },
    # The load balancer estimates the runtime for each query and is used by CompassApp to best leverage parallelism.
    { type = "load_balancer", weight_heuristic = { type = "haversine" } },
]
output_plugins = [
    # The traversal plugin appends various items to the result.
    { type = "traversal", route = "geo_json", geometry_input_file = "edges-geometries-enumerated.txt.gz" },
    # The uuid plugin adds a map specific id (like Open Street Maps Nodes) onto the compass verticies
    { type = "uuid", uuid_input_file = "vertices-uuid-enumerated.txt.gz" },
]
```

## Traversal Models

Traversal models are what the application uses when computing a path through the graph.
The models can use things like road speed to compute the shortest time route or vehicle energy consumption to compute a route that uses the least energy.
Here are the default traversal models that come with the `CompassApp`:

### Distance

The distance traversal model is a very simple model that just uses distance for computing a route, producing the route that has the shortest distance.

```toml
[traversal]
type = "distance"
distance_unit = "miles"
```

### Speed Table

The speed table traversal model uses a speed lookup table to compute the fastest (or shortest time) route.

```toml
[traversal]
type = "speed_table"
speed_table_input_file = "edges-posted-speed-enumerated.txt.gz"
speed_unit = "kilometers_per_hour"
distance_unit = "miles"
time_unit = "minutes"
```

### Energy Model

The energy model computes energy (with a routee-powertrain vehicle model) and speed over an edge.

```toml
[traversal]
type = "energy_model"
# the units of the speed table
time_model_speed_unit = "kilometers_per_hour"
# the file that has grades for each edge in the graph
grade_table_input_file = "edges-grade-enumerated.txt.gz"
# the units of the grade table
grade_table_grade_unit = "decimal"

# the internal units of the energy model
time_unit = "minutes"
distance_unit = "miles"

# Here we specify the time model to use for the energy model
[traversal.time_model]
type = "speed_table"
# the file that has speeds for each edge in the graph
speed_table_input_file = "edges-posted-speed-enumerated.txt.gz"
# the units of the speed table
speed_unit = "kilometers_per_hour"

# the internal units of the speed table
distance_unit = "miles"
time_unit = "minutes"

# here, we specify which vehicles to make available at query time
# if you wanted to add more models, you would make a new [[traversal.vehicles]] section.
[[traversal.vehicles]]
# the name of the model that can be passed in from a query as "model_name"
name = "2012_Ford_Focus"
# the type of the vehicle, currently either:
# - "ice" i.e. Internal Combustion Engine (ICE)
# - "bev" i.e. Battery Electric Vehicle (BEV)
# - "phev" i.e. Plug-in Hybrid Electric Vehicle (PHEV)
type = "ice"
# the file for the routee-powertrain model
model_input_file = "models/2012_Ford_Focus.bin"
# the units of what the routee-powertrain model expects speed to be in
speed_unit = "miles_per_hour"
# the units of what the routee-powertrain model expects grade to be in
grade_unit = "decimal"
# the units of what the routee-powertrain model outputs for energy
energy_rate_unit = "gallons_gasoline_per_mile"
# the "best case" energy rate for this particular vehicle (something like highway mpg) that's used in the a-star algorithm
ideal_energy_rate = 0.02857143
# A real world adjustment factor for things like temperature and auxillary loads
real_world_energy_adjustment = 1.166

# what underlying machine learn framework to use [smartcore | interpolate | onnx]
# in this case we use a model that interpolates the underlying model type over a regular grid
[traversal.vehicles.model_type.interpolate]
underlying_model_type = "smartcore"
speed_lower_bound = 0
speed_upper_bound = 100
speed_bins = 101
grade_lower_bound = -0.2
grade_upper_bound = 0.2
grade_bins = 41

```

## Plugins

Input and output plugins are used to modify the queries and the results respectively.
Both queries and results are valid json objects and so a plugin takes in a json object and returns a json object.

## Input Plugins

Here are the default input plugins that are provided:

### Grid Search

The grid search plugin allows you to specify a `grid_search` key in the query and it will generate multiple queries from those parameters. For example, if you had a query:

```json
{
  "origin_name": "Government Center Station",
  "destination_name": "Cannonball Creek Brewery",
  "origin_x": -105.200146,
  "origin_y": 39.72657,
  "destination_x": -105.234964,
  "destination_y": 39.768477,
  "model_name": "2016_TOYOTA_Camry_4cyl_2WD",
  "grid_search": {
    "test_cases": [
      {
        "name": "shortest_time",
        "weights": {
          "distance": 0.0,
          "time": 1.0,
          "energy_liquid": 0.0,
          "energy_electric": 0.0
        }
      },
      {
        "name": "least_energy",
        "weights": {
          "distance": 0.0,
          "time": 0.0,
          "energy_liquid": 1.0,
          "energy_electric": 1.0
        }
      }
    ]
  }
}
```

The grid search plugin would take this single query and generate two queries that would be fed into the application:

```json
[
  {
    "origin_name": "Government Center Station",
    "destination_name": "Cannonball Creek Brewery",
    "origin_x": -105.200146,
    "origin_y": 39.72657,
    "destination_x": -105.234964,
    "destination_y": 39.768477,
    "model_name": "2016_TOYOTA_Camry_4cyl_2WD",
    "name": "shortest_time",
    "weights": {
      "distance": 0.0,
      "time": 1.0,
      "energy_liquid": 0.0,
      "energy_electric": 0.0
    }
  },
  {
    "origin_name": "Government Center Station",
    "destination_name": "Cannonball Creek Brewery",
    "origin_x": -105.200146,
    "origin_y": 39.72657,
    "destination_x": -105.234964,
    "destination_y": 39.768477,
    "name": "least_energy",
    "model_name": "2016_TOYOTA_Camry_4cyl_2WD",
    "weights": {
      "distance": 0.0,
      "time": 0.0,
      "energy_liquid": 1.0,
      "energy_electric": 1.0
    }
  }
]
```

```toml
[[plugin.input_plugins]]
type = "grid_search"
```

### Vertex RTree

The vertex RTree plugin uses an RTree to match coordiantes to graph verticies.

For example, if you specify your query origin and destination as lat/lon coordinates (i.e. `origin_x`, `origin_y`) we need a way to match this to the graph and then insert an `origin_vertex` or a `destination_vertex` into the query. Those two fields are what the application expects when conducting a search.

```toml
[[plugin.input_plugins]]
type = "vertex_rtree"
# the vertices of the graph; enumerated to match the index of the graph vertex file
vertices_input_file = "vertices-compass.csv.gz"
```

### Edge RTree

The edge RTree plugin uses an RTree to match coordiantes to graph edges.

For example, if you specify your query origin and destination as lat/lon coordinates (i.e. `origin_x`, `origin_y`) we need a way to match this to the graph and then insert an `origin_edge` or a `destination_edge` into the query.

The Edge RTree has some additional paramters as comparted to the Vertex RTree.
Specifically, the Edge RTree takes in geomteries for each edge as well as road classes for each edge.
It uses the geometries for computing the distance between the incoming points and the edge.

In addition, it uses the road classes to optionally filter out road classes that need to be excluded at query time by supplying a "road_classes" argument to the query with a list of strings to match against.

```toml
[[plugin.input_plugins]]
type = "edge_rtree"
# geometries for each edge; enumerated to match the index of the graph edge file
geometry_input_file = "edge-geometries.csv.gz"
# road classes for each edge; enumerated to match the index of the graph edge file
road_class_input_file = "road-classes.csv.gz"
# how far around the point to search (smaller could improve performance but too small might result in no matches)
distance_tolerance = 100
# unit of the distance tolerance
distance_unit = "meters"
```

### Load Balancer

The load balancer plugin estimates the runtime for each query. That information is used by `CompassApp` in order to best leverage parallelism.

For example, we have configured a parallelism of 2 and have 4 queries, but one query is a cross-country trip and will take a very long time to run.
With the load balancer plugin, Compass will identify this and bundle the three smaller queries together:

```
naive = [[long, short], [short, short]]
balanced = [[long], [short, short, short]]
```

```toml
[[plugin.input_plugins]]
type = "load_balancer"
# method for estimating query runtime, in this case haversine distance in kilometers.
# this heuristic only works for trips with origin/destination pairs.
weight_heuristic = { type = "haversine" }
```

if a user has fields on their queries that can be used directly or mapped to weight values, they may use
the custom weight heuristic. this numeric example expects a field `my_weight_value: float` on each query:

```toml
[[plugin.input_plugins]]
type = "load_balancer"
[plugin.input_plugins.weight_heuristic]
type = "custom"
[plugin.input_plugins.weight_heuristic.custom_weight_type]
type = "numeric"
column_name = "my_weight_value"
```

categorical fields are also supported by providing a mapping. this example expects a `mode` field
and uses values `[walk, bike, drive]` to map to weight values of 1, 10, and 100, for example
based on observed search sizes for each travel mode.

```toml
[[plugin.input_plugins]]
type = "load_balancer"
[plugin.input_plugins.weight_heuristic]
type = "custom"
[plugin.input_plugins.weight_heuristic.custom_weight_type]
type = "categorical"
column_name = "mode"
default = 1
mapping = { "walk" = 1, "bike" = 10, "drive" = 100 }
```

## Output Plugins

Here are the default output plugins that are provided:

### Traversal

A plugin that appends various items to the result.

```toml
[[plugin.output_plugins]]
type = "traversal"
route = "geo_json"
tree = "geo_json"
geometry_input_file = "edges-geometries-enumerated.txt.gz"
```

The `route` key will add route information to the result depending on the type.
The `tree` key will add search tree information to the result depending on the type (be aware that this could be very large for searches that span a large geographical distance).

Both the `route` and the `tree` key are optional and if omitted, the plugin will not append anything for it. In addition both keys can be specified in the following formats:

- "json": non-geometry output writing traversal metrics (cost, state) as JSON for a route or a tree
- "wkt": outputs a LINESTRING for a route, or a MULTILINESTRING for a tree
- "geo_json": annotated geometry data as a FeatureCollection of LineStrings with properties assigned from traversal metrics
