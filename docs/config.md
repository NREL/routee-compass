# App Config

Each `CompassApp` instance is defined by a configuration toml file.
The configuration file specifies things like "Which traversal model should I use, and what are its parameters?" and "Which data sources should I use?".

If you follow the [open street maps example](notebooks/open_street_maps_example.ipynb), the code will produce a few configuration files in the `golden_co/` folder. Let's take a look at the `osm_default_energy.toml` file.
We added some annotations to describe the different sections:

```toml
# how many threads should a CompassApp use to process queries?
parallelism = 2

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
# the energy_model can compute routes using time or energy as costs
type = "energy_model"
# speeds for each edge in the graph
speed_table_input_file = "edges-posted-speed-enumerated.txt.gz"
# the units of the above speed table
speed_table_speed_unit = "kilometers_per_hour"
# grades for each edge in the graph
grade_table_input_file = "edges-grade-enumerated.txt.gz"
# the units of the above graph table
grade_table_grade_unit = "decimal"
# the units of what the traversal model outputs for time
output_time_unit = "minutes"
# the units of what the traversal model outputs for distance
output_distance_unit = "miles"

# here, we specify which vehicles to make available at query time
# if you wanted to add more models, you would make a new [[traversal.vehicles]] section.
[[traversal.vehicles]]
# the name of the model that can be passed in from a query as "model_name"
name = "2012_Ford_Focus"
# the type of the vehicle, currently either:
# - "single_fuel" i.e. Internal Combustion Engine (ICE) or Battery Electric (BEV)
# - "duel_fuel" i.e. Plug in Hybrid Electric (PHEV)
type = "single_fuel"
# the file for the routee-powertrain model
model_input_file = "models/2012_Ford_Focus.bin"
# what underlying machine learn framework to use [smartcore | interpolate]
model_type = "smartcore"
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


# which plugins should be activated?
[plugin]

# input plugins get applied to a query before it gets executed
[[plugin.input_plugins]]
# the grid search plugin allows for multiple queries to be generated from one query with the "grid_search" key
type = "grid_search"

[[plugin.input_plugins]]
# a vertex based RTree for matching incoming x/y coordinates to a graph vertex
type = "vertex_rtree"
# a file with all the graph verticies
vertices_input_file = "vertices-compass.csv.gz"

# output plugs get applied to the result of running a query
[[plugin.output_plugins]]
# summarize the results of the search
type = "summary"

[[plugin.output_plugins]]
# a plugin that gathers traversal specific outputs like the route geometry
type = "traversal"
# return the full route in a geojson format
route = "geo_json"
# return the full search tree in a geojson format
tree = "geo_json"
# geometry objects for all the edges in the graph
geometry_input_file = "edges-geometries-enumerated.txt.gz"

[[plugin.output_plugins]]
# append an map specific id(like Open Street Maps Nodes) onto the compass verticies (which use a simple integer internal index)
type = "uuid"
# a file with ids for each vertex in the graph
uuid_input_file = "vertices-uuid-enumerated.txt.gz"
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
output_distance_unit = "miles"
output_time_unit = "minutes"
```

### Energy Model

The energy model computes energy (with a routee-powertrain vehicle model) and speed over an edge. This model also takes in an additional query argument `energy_cost_coefficient` that specifies how much to value energy in the resulting cost. For example, an `energy_cost_coefficient` of `0.0` would not value energy at all and would return a pure shortest time route whereas an `energy_cost_coefficient` of `1.0` would not value time at all and would return a pure least energy route.

```toml
[traversal]
# the energy_model can compute routes using time or energy as costs
type = "energy_model"
# speeds for each edge in the graph
speed_table_input_file = "edges-posted-speed-enumerated.txt.gz"
# the units of the above speed table
speed_table_speed_unit = "kilometers_per_hour"
# grades for each edge in the graph
grade_table_input_file = "edges-grade-enumerated.txt.gz"
# the units of the above graph table
grade_table_grade_unit = "decimal"
# the units of what the traversal model outputs for time
output_time_unit = "minutes"
# the units of what the traversal model outputs for distance
output_distance_unit = "miles"

# here, we specify which vehicles to make available at query time
# if you wanted to add more models, you would make a new [[traversal.vehicles]] section.
[[traversal.vehicles]]
# the name of the model that can be passed in from a query as "model_name"
name = "2012_Ford_Focus"
# the type of the vehicle, currently either:
# - "single_fuel" i.e. Internal Combustion Engine (ICE) or Battery Electric (BEV)
# - "duel_fuel" i.e. Plug in Hybrid Electric (PHEV)
type = "single_fuel"
# the file for the routee-powertrain model
model_input_file = "models/2012_Ford_Focus.bin"
# what underlying machine learn framework to use [smartcore | interpolate]
model_type = "smartcore"
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

# An optional cache that will keep track of recent energy values with the same input.
# If this is ommitted from the config, the model will compute the energy for every link in a search.
# See note below for some considerations when using this.
[traversal.vehicles.float_cache_policy]
cache_size = 10_000
key_precisions = [
    2, # speed goes from 71.23 to 7123
    4, # grade goes from 0.123 (decimal) to 1230 or 123 (millis) to 1230000
]
```

```{note}
When using the float cache it's possible that you might get slightly different final energy values versus the same query run with no caching.

The reason for this is how the input floating point values get converted into an integer for storage in the cache.

If your key precision for a grade value is 4, an incoming value of 0.123456 would get converted into 1234 and stored in the cache as such.
This is done to make sure we're actually getting cache hits and improving performance.
If we used a precision of 10, there might not be many other links in the road network that share the same exact properties at that resolution.
But, the tradeoff here is that if you used a key precision of 1, grade values of 0.14 and 0.05 would both result in the integer 1 being stored in the cache.
This would render grades of 5% and 14% to be equal to each other from an energy perspective and they are clearly not.

So, usage of this cache can result in improved runtimes for the energy traversal model but the user should make sure the precision values are appropriate for the application.
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
  "grid_search": {
    "energy_cost_coefficient": [0.0, 1.0]
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
    "energy_cost_coefficient": 0.0
  },
  {
    "origin_name": "Government Center Station",
    "destination_name": "Cannonball Creek Brewery",
    "origin_x": -105.200146,
    "origin_y": 39.72657,
    "destination_x": -105.234964,
    "destination_y": 39.768477,
    "energy_cost_coefficient": 1.0
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

### Summary

A plugin adds a simple `traversal_summary` section to the result.

```toml
[[plugin.output_plugins]]
type = "summary"
```

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

### To Disk

The `to_disk` plugin writes the results to a specified output file rather than returning them when the `run` method is called.

This plugin writes the results in newline delimited JSON.

```toml
[[plugin.output_plugins]]
type = "to_disk"

# where to write the results
# relative to where the application is being run
output_file = "result.json"
```
