# Query

The query is what you use to specify the parameters for a search (or multiple searches)
and is in a json format.

## Basic

Here's a very simple example query:

```json
{
  "origin_name": "Government Center Station",
  "destination_name": "Cannonball Creek Brewery",
  "origin_x": -105.200146,
  "origin_y": 39.72657,
  "destination_x": -105.234964,
  "destination_y": 39.768477
}
```

In this example, note that the keys `origin_name` and `destination_name` are completely optional and are only used for documentation purposes.
The application does not use them but they do get passed through to the result.
You can provide any arbitrary key if you want to pass information through.

The remaining keys are used to define where we should start and end our search:

- `origin_x`: The longitude of the origin coordinate
- `origin_y`: The latitude of the origin coordinate
- `destination_x`: The longitude of the origin coordinate
- `destination_y`: The latitude of the origin coordinate

## Multiple Queries

In addition to a single query, you can also pass multiple queries into the app and it will run them in parallel according to the `parallelism` setting in the [config](config)

Here's an example:

```json
[
  {
    "origin_x": -105.200146,
    "origin_y": 39.72657,
    "destination_x": -105.234964,
    "destination_y": 39.768477
  },
  {
    "origin_x": -105.234964,
    "origin_y": 39.768477,
    "destination_x": -105.200146,
    "destination_y": 39.72657
  }
]
```

## Grid Search

If you have the `grid_search` input plugin enabled, you can also provide a `grid_search` key that the plugin will use to generate multiple queries from a single query.

For example, if you're using the `speed_grade_energy_model` traversal model, it takes an additional key `energy_cost_coefficient` and so if you wanted to sweep this value you could do something like:

```json
query = {
    "origin_name": "Government Center Station",
    "destination_name": "Cannonball Creek Brewery",
    "origin_x": -105.200146,
    "origin_y": 39.726570,
    "destination_x": -105.234964,
    "destination_y": 39.768477,
    "grid_search": {
        "energy_cost_coefficient": [0.0, 0.5, 1.0]
    }
}
```

This would generate 3 unique queries with the same origin and destination parameters and the appropriate `energy_cost_coefficient`.

The `grid_search` plugin also allows you to provide objects if you wanted to pass aribrary keys through to the result:

```json
query = {
    "origin_name": "Government Center Station",
    "destination_name": "Cannonball Creek Brewery",
    "origin_x": -105.200146,
    "origin_y": 39.726570,
    "destination_x": -105.234964,
    "destination_y": 39.768477,
    "grid_search": {
        "test_cases": [
            {"energy_cost_coefficient": 0.0, "name": "Shortest Time"},
            {"energy_cost_coefficient": 0.5, "name": "Time/Energy Mix"},
            {"energy_cost_coefficient": 1.0, "name": "Least Energy"},
        ]
    }
}
```

This would similarily produce three unique queries with the appropriate `energy_cost_coefficient` and `name` fields.
