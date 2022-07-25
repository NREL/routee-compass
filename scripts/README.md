# Scripts

## `download_road_map.py`
You can use this script to download a road network for computing energy routes. 

For example, you can download the road network around downtown denver by using the geojson file in the tests folder:

```bash
python download_road_map.py ../tests/test_assets/downtown_denver.geojson downtown_denver_road_map.json
```

Usage:
```
usage: download_road_map.py [-h] geofence_file output_file

Download a road map from Trolley

positional arguments:
  geofence_file  Geofence file to use
  output_file    Output file to write to

optional arguments:
  -h, --help     show this help message and exit
```

## `plot_routes.py`
You can use this script to plot the shortest energy route for a gasoline vehicle.

For example, using the network downloaded above, we can route between two coordinates:

```
python plot_routes.py downtown_denver_road_map.json \
    --origin-lat 39.754372 \
    --origin-lon -104.994300 \
    --dest-lat 39.779098 \
    --dest-lon -104.951241
```

Usage:
```
usage: plot_routes.py [-h] [--origin-lat ORIGIN_LAT] [--origin-lon ORIGIN_LON] [--dest-lat DEST_LAT] [--dest-lon DEST_LON]
                      [--output OUTPUT]
                      road_network_file

Plot routee-compass routes

positional arguments:
  road_network_file     Road network file to use

optional arguments:
  -h, --help            show this help message and exit
  --origin-lat ORIGIN_LAT
                        Origin latitude
  --origin-lon ORIGIN_LON
                        Origin longitude
  --dest-lat DEST_LAT   Destination latitude
  --dest-lon DEST_LON   Destination longitude
  --output OUTPUT       Output file
```